use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::config::Config;
use crate::device;
use crate::ui;

/// How long to wait for the browser to come back before giving up. Long enough
/// to type a password and answer a second factor, short enough that a closed
/// tab does not leave a listener open forever.
const WAIT: Duration = Duration::from_secs(180);

pub const DEFAULT_SERVER: &str = "https://sablier.facile.studio";

#[derive(Deserialize)]
struct ExchangeResponse {
    token: String,
}

/// Sign in and write the token to the config file.
///
/// The default is the device grant: the CLI prints an approval link, the human
/// approves it in any browser, and nothing listens on this machine, so it works
/// over SSH. `listener` opts into the loopback redirect instead, which is also
/// the automatic fallback for a server that has not mounted the device exchange.
pub async fn run(server: Option<String>, listener: bool) -> Result<()> {
    let server = resolve_server(server)?;
    let api = api_base(&server);

    if !sso_enabled(&api).await? {
        bail!("this server has no single sign-on, put a token from Profile > API Token in ~/.sablier.yml");
    }

    let token = if listener || !device::exchange_mounted(&api).await? {
        if !listener {
            ui::warn(
                "This server has no device sign-in yet, falling back to the browser redirect.",
            );
        }
        loopback(&api).await?
    } else {
        device::exchange(&api, &device::access_token(has_display()).await?).await?
    };
    let config = Config {
        server_url: api,
        token,
    };
    config.save()?;

    ui::success(&format!("Signed in to {server}"));
    Ok(())
}

/// The loopback flow. The CLI never sees the identity provider: it opens a
/// port the kernel picks, so two shells can log in at once, sends the browser to
/// the API with that port and a nonce attached, and the API redirects back with
/// a one-time code valid for sixty seconds. The nonce is what tells the
/// listener its own callback from one another local process sent.
async fn loopback(api: &str) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("cannot open a loopback port to receive the login")?;
    let port = listener.local_addr()?.port();
    let state = nonce();

    let url = format!("{api}/auth/oidc?flow=cli&port={port}&cli_state={state}");
    ui::step("Opening your browser to sign in...");
    if open_browser(&url).is_err() {
        ui::hint(&format!(
            "Could not open a browser, open this URL in one: {url}"
        ));
    }

    let code = match tokio::time::timeout(WAIT, wait_for_code(listener, &state)).await {
        Ok(result) => result?,
        Err(_) => bail!("timed out waiting for the browser, run `sablier login` again"),
    };
    exchange(api, &code).await
}

/// Whether opening a browser can work here. Over SSH there is no display, and
/// `xdg-open` would reach for a text browser that takes over the terminal.
fn has_display() -> bool {
    if cfg!(target_os = "linux") {
        return std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some();
    }
    true
}

/// api_base is the value the rest of the CLI expects in server_url: the API
/// root, including `/api`.
///
/// Every request is built by appending a path to it verbatim — `api::url` is a
/// bare format! — so a `server_url` of `https://sablier.example.com` silently
/// produces `https://sablier.example.com/time-entries` and 404s. Accepting the
/// bare host on the command line and normalising here is what makes
/// `sablier login https://sablier.example.com` do the obvious thing.
fn api_base(server: &str) -> String {
    let trimmed = server.trim_end_matches('/');
    if trimmed.ends_with("/api") {
        return trimmed.to_string();
    }
    format!("{trimmed}/api")
}

fn resolve_server(server: Option<String>) -> Result<String> {
    if let Some(server) = server {
        return Ok(server);
    }
    if let Ok(existing) = Config::load() {
        if !existing.server_url.is_empty() {
            return Ok(existing.server_url);
        }
    }
    Ok(DEFAULT_SERVER.to_string())
}

async fn sso_enabled(api: &str) -> Result<bool> {
    #[derive(Deserialize)]
    struct Discovery {
        #[serde(default)]
        oidc_enabled: bool,
    }
    let response = reqwest::Client::new()
        .get(format!("{api}/auth/config"))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .with_context(|| format!("cannot reach {api}"))?;
    let discovery: Discovery = response
        .json()
        .await
        .context("the server did not answer like a Sablier server")?;
    Ok(discovery.oidc_enabled)
}

/// wait_for_code serves exactly one request: the redirect the API sends the
/// browser to. It parses the request line rather than pulling in an HTTP
/// server, because the only request it will ever see is a GET it constructed
/// the URL for itself.
async fn wait_for_code(listener: TcpListener, expected_state: &str) -> Result<String> {
    loop {
        let (mut stream, _) = listener.accept().await?;

        let mut buffer = [0u8; 2048];
        let read = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..read]);
        let target = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("");

        // A browser asks for /favicon.ico unprompted; answering it as if it
        // were the callback would fail the login for no reason.
        let Some(code) = query_value(target, "code") else {
            respond(&mut stream, "404 Not Found", "Not the login redirect.").await?;
            continue;
        };

        // A callback carrying a code but the wrong nonce is not noise, it is
        // somebody else's. Aborting is the point of sending one.
        if query_value(target, "state").as_deref() != Some(expected_state) {
            respond(
                &mut stream,
                "400 Bad Request",
                "The callback did not match this login attempt. Run `sablier login` again.",
            )
            .await?;
            bail!(
                "the sign-in callback did not match this login attempt, run `sablier login` again"
            );
        }

        respond(&mut stream, "200 OK", "Sablier has your login.").await?;
        return Ok(code);
    }
}

/// A nonce the server echoes back, so the listener can recognise its own
/// callback. `/dev/urandom` keeps this free of a dependency; the fallback only
/// ever runs where that file is missing, and a guessable nonce still beats none.
fn nonce() -> String {
    use std::io::Read;

    let mut bytes = [0u8; 16];
    let seeded = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .is_ok();

    if !seeded {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let mixed = nanos ^ ((std::process::id() as u128) << 96);
        bytes.copy_from_slice(&mixed.to_le_bytes());
    }

    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn query_value(target: &str, key: &str) -> Option<String> {
    let query = target.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name != key || value.is_empty() {
            return None;
        }
        Some(percent_decode(value))
    })
}

/// percent_decode handles what a one-time code can actually contain. porte's
/// codes are base64url, so nothing needs escaping — but the value arrives
/// through a URL and assuming it is clean is how the one code with a `+` in it
/// fails a year from now.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

async fn respond(stream: &mut tokio::net::TcpStream, status: &str, message: &str) -> Result<()> {
    let (heading, hint) = if status.starts_with("200") {
        (
            "Signed in",
            "You can close this tab and go back to your terminal.",
        )
    } else {
        (
            "Not signed in",
            "The login started in your terminal is still waiting.",
        )
    };
    let body = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{heading} · Sablier</title>\
         <style>:root{{color-scheme:light dark;--bg:#fafafa;--fg:#18181b;--muted:#71717a;--card:#fff}}\
         @media (prefers-color-scheme:dark){{:root{{--bg:#09090b;--fg:#fafafa;--muted:#a1a1aa;--card:#18181b}}}}\
         body{{margin:0;min-height:100dvh;display:flex;align-items:center;justify-content:center;padding:1.5rem;\
         background:var(--bg);color:var(--fg);font:16px/1.6 ui-sans-serif,system-ui,sans-serif}}\
         main{{width:100%;max-width:26rem;text-align:center}}\
         .brand{{font-size:1.125rem;font-weight:650;margin-bottom:2rem}}\
         .card{{background:var(--card);border-radius:.75rem;padding:1.75rem}}\
         h1{{margin:0;font-size:1.25rem}}p{{margin:.5rem 0 0;font-size:.875rem;color:var(--muted)}}\
         .hint{{margin-top:1.5rem;font-size:.8125rem;color:var(--muted)}}</style>\
         <main><div class=\"brand\">Sablier</div>\
         <div class=\"card\"><h1>{heading}</h1><p>{message}</p></div>\
         <p class=\"hint\">{hint}</p></main>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn exchange(api: &str, code: &str) -> Result<String> {
    let url = format!("{api}/auth/oidc/exchange");
    let response = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({ "code": code }))
        .send()
        .await
        .context("cannot reach the server to exchange the login code")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("the server refused the login code ({status}): {body}");
    }
    let exchanged: ExchangeResponse = response
        .json()
        .await
        .context("the server's answer to the code exchange was not what was expected")?;
    if exchanged.token.is_empty() {
        bail!("the server returned an empty token");
    }
    Ok(exchanged.token)
}

pub fn open_browser(url: &str) -> Result<()> {
    let command = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let status = std::process::Command::new(command)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if !status.success() {
        bail!("browser command failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // server_url is appended to verbatim by api::url, so it has to be the API
    // root. Getting this wrong sends every later request to a 404 and the
    // login itself would still appear to succeed.
    #[test]
    fn api_base_normalises_what_a_human_would_type() {
        for input in [
            "https://sablier.facile.studio",
            "https://sablier.facile.studio/",
            "https://sablier.facile.studio/api",
            "https://sablier.facile.studio/api/",
        ] {
            assert_eq!(
                api_base(input),
                "https://sablier.facile.studio/api",
                "{input}"
            );
        }
    }

    #[test]
    fn the_code_is_read_from_the_redirect_and_nothing_else_is() {
        assert_eq!(
            query_value("/?code=abc123", "code").as_deref(),
            Some("abc123")
        );
        assert_eq!(
            query_value("/?state=x&code=abc123", "code").as_deref(),
            Some("abc123")
        );
        assert_eq!(query_value("/favicon.ico", "code"), None);
        assert_eq!(query_value("/?code=", "code"), None);
        assert_eq!(query_value("/", "code"), None);
    }

    #[test]
    fn percent_decoding_survives_an_escaped_code() {
        assert_eq!(percent_decode("a-b_c"), "a-b_c");
        assert_eq!(percent_decode("a%2Bb"), "a+b");
        assert_eq!(percent_decode("a+b"), "a b");
    }

    #[test]
    fn nonce_is_hex_and_does_not_repeat() {
        let first = nonce();
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, nonce());
    }

    #[tokio::test]
    async fn a_callback_with_the_wrong_nonce_is_refused() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            stream
                .write_all(b"GET /?code=stolen&state=not-ours HTTP/1.1\r\n\r\n")
                .await
                .unwrap();
            let mut sink = Vec::new();
            let _ = stream.read_to_end(&mut sink).await;
        });

        assert!(wait_for_code(listener, "ours").await.is_err());
    }

    #[tokio::test]
    async fn a_callback_with_the_right_nonce_is_accepted() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            stream
                .write_all(b"GET /favicon.ico HTTP/1.1\r\n\r\n")
                .await
                .unwrap();
            let mut sink = Vec::new();
            let _ = stream.read_to_end(&mut sink).await;

            let mut second = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            second
                .write_all(b"GET /?code=real&state=ours HTTP/1.1\r\n\r\n")
                .await
                .unwrap();
            let mut sink = Vec::new();
            let _ = second.read_to_end(&mut sink).await;
        });

        assert_eq!(wait_for_code(listener, "ours").await.unwrap(), "real");
    }
}
