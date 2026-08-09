use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::config::Config;
use crate::ui;

/// How long to wait for the browser to come back before giving up. Long enough
/// to type a password and answer a second factor, short enough that a closed
/// tab does not leave a listener open forever.
const WAIT: Duration = Duration::from_secs(180);

#[derive(Deserialize)]
struct ExchangeResponse {
    token: String,
}

/// Sign in through the browser and write the token to the config file.
///
/// The whole exchange belongs to the server: the CLI never sees the identity
/// provider, never handles a password, and never holds an authorization code
/// that is worth anything on its own. It opens a loopback port, sends the
/// browser to the API with that port attached, and the API — after the provider
/// has done its part — redirects back to the port with a one-time code that is
/// valid for sixty seconds and works once.
pub async fn run(server: Option<String>) -> Result<()> {
    let server = resolve_server(server)?;
    let api = api_base(&server);

    // Port zero asks the kernel for a free one, so two shells can log in at
    // the same time without agreeing on anything.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("cannot open a loopback port to receive the login")?;
    let port = listener.local_addr()?.port();

    let url = format!(
        "{}/api/auth/oidc?flow=cli&port={}",
        server.trim_end_matches('/'),
        port
    );
    ui::step(&format!("Opening {url}"));
    if open_browser(&url).is_err() {
        ui::hint("Could not open a browser — paste that URL into one.");
    }

    let code = match tokio::time::timeout(WAIT, wait_for_code(listener)).await {
        Ok(result) => result?,
        Err(_) => bail!("timed out waiting for the browser, run `sablier login` again"),
    };

    let token = exchange(&server, &code).await?;
    let config = Config {
        server_url: server,
        token,
    };
    config.save()?;

    ui::success(&format!(
        "Signed in. Token saved to {}",
        Config::path()?.display()
    ));
    Ok(())
}

/// api_base is the value the rest of the CLI expects in server_url: the API
/// root, including `/api`.
///
/// Every request is built by appending a path to it verbatim — `api::url` is a
/// bare format! — so a `server_url` of `https://sablier.example.com` silently
/// produces `https://sablier.example.com/time-entries` and 404s. Accepting the
/// bare host on the command line and normalising here is what makes
/// `--server https://sablier.example.com` do the obvious thing.
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
    bail!("no server known — run `sablier login --server https://sablier.example.com`")
}

/// wait_for_code serves exactly one request: the redirect the API sends the
/// browser to. It parses the request line rather than pulling in an HTTP
/// server, because the only request it will ever see is a GET it constructed
/// the URL for itself.
async fn wait_for_code(listener: TcpListener) -> Result<String> {
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

        respond(
            &mut stream,
            "200 OK",
            "Signed in. You can close this tab and go back to your terminal.",
        )
        .await?;
        return Ok(code);
    }
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
    let body = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Sablier</title>\
         <body style=\"font:16px/1.5 system-ui,sans-serif;margin:4rem auto;max-width:32rem;padding:0 1rem\">\
         <h1>Sablier</h1><p>{message}</p>"
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

fn open_browser(url: &str) -> Result<()> {
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
}
