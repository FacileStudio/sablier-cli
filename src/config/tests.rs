use super::*;

mod fake;
use fake::FakeVault;

fn scratch(name: &str, contents: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("sablier-cli-{}-{name}.yml", std::process::id()));
    std::fs::write(&path, contents).unwrap();
    path
}

const SIGNED_IN: &str = "server_url: https://sablier.test/api\ntoken: secret\n";

#[test]
fn a_file_token_moves_to_the_keyring_and_leaves_the_file() {
    let path = scratch("migrate", SIGNED_IN);
    let vault = FakeVault::new(true, true);

    let config = Config::load_from(&path, &vault).unwrap();

    assert_eq!(config.token, "secret");
    assert_eq!(vault.token.borrow().as_deref(), Some("secret"));
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(!on_disk.contains("secret"), "file kept {on_disk}");
    assert!(on_disk.contains("server_url: https://sablier.test/api"));
    assert_eq!(Config::load_from(&path, &vault).unwrap().token, "secret");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn without_a_keyring_the_token_stays_in_the_file() {
    let path = scratch("fallback", SIGNED_IN);
    let vault = FakeVault::new(false, false);

    let config = Config::load_from(&path, &vault).unwrap();

    assert_eq!(config.token, "secret");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), SIGNED_IN);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn a_keyring_that_drops_the_write_never_costs_the_file_copy() {
    let path = scratch("dropped", SIGNED_IN);
    let vault = FakeVault::new(true, false);

    assert_eq!(Config::load_from(&path, &vault).unwrap().token, "secret");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), SIGNED_IN);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn login_writes_the_token_to_the_keyring_or_else_the_file() {
    let config = Config {
        server_url: "https://sablier.test/api".into(),
        token: "secret".into(),
    };
    let path = scratch("save", "");

    config.save_to(&path, &FakeVault::new(true, true)).unwrap();
    assert!(!std::fs::read_to_string(&path).unwrap().contains("secret"));

    config
        .save_to(&path, &FakeVault::new(false, false))
        .unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), SIGNED_IN);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn logout_clears_the_keyring_and_the_file_but_keeps_the_server() {
    let path = scratch("logout", SIGNED_IN);
    let vault = FakeVault::new(true, true);
    vault.set("secret").unwrap();

    assert!(Config::clear_at(&path, &vault).unwrap());

    assert!(vault.token.borrow().is_none());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "server_url: https://sablier.test/api\n"
    );
    assert!(!Config::clear_at(&path, &vault).unwrap());
    assert!(!Config::clear_at(&path, &FakeVault::new(false, false)).unwrap());
    std::fs::remove_file(&path).unwrap();
}
