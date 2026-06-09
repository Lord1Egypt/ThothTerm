use thothterm_web3::{Web3Client, validate_address, hex_to_u64, wei_to_eth_string, is_ens_name, detect_web3_project};
use thothterm_web3::rpc::chain_name;
use thothterm_web3::detector::ProjectType;
use mockito::{Server, ServerGuard};
use tempfile::tempdir;
use std::fs;

fn rpc_response(result: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","id":1,"result":"{}"}}"#, result)
}

fn make_client(server: &ServerGuard) -> Web3Client {
    Web3Client::new(server.url())
}

// ── Address validation ────────────────────────────────────────────────────────

#[test]
fn valid_ethereum_address_passes() {
    assert!(validate_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_ok());
}

#[test]
fn address_missing_0x_prefix_rejected() {
    assert!(validate_address("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_err());
}

#[test]
fn address_wrong_length_rejected() {
    assert!(validate_address("0xshort").is_err());
}

#[test]
fn address_non_hex_chars_rejected() {
    assert!(validate_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG").is_err());
}

// ── Hex conversion ────────────────────────────────────────────────────────────

#[test]
fn hex_to_u64_with_prefix() {
    assert_eq!(hex_to_u64("0x1").unwrap(), 1);
    assert_eq!(hex_to_u64("0xff").unwrap(), 255);
    assert_eq!(hex_to_u64("0x1e846f").unwrap(), 0x1e846f);
}

#[test]
fn hex_to_u64_without_prefix() {
    assert_eq!(hex_to_u64("1a").unwrap(), 26);
}

#[test]
fn wei_to_eth_formatting() {
    assert_eq!(wei_to_eth_string(1_000_000_000_000_000_000), "1.0000 ETH");
    assert_eq!(wei_to_eth_string(0), "0.0000 ETH");
}

// ── ENS detection ─────────────────────────────────────────────────────────────

#[test]
fn ens_name_detection() {
    assert!(is_ens_name("vitalik.eth"));
    assert!(is_ens_name("thoth.eth"));
    assert!(!is_ens_name("notens.com"));
    assert!(!is_ens_name("0xabc123"));
    assert!(!is_ens_name(".eth")); // too short
}

// ── Chain names ───────────────────────────────────────────────────────────────

#[test]
fn known_chain_names() {
    assert_eq!(chain_name(1), "Ethereum Mainnet");
    assert_eq!(chain_name(137), "Polygon");
    assert_eq!(chain_name(31337), "Anvil/Hardhat Local");
    assert_eq!(chain_name(99999), "Unknown Network");
}

// ── RPC health check ──────────────────────────────────────────────────────────

#[tokio::test]
async fn health_check_success() {
    let mut server = Server::new_async().await;
    // chain_id response
    let mock = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(rpc_response("0x1")) // mainnet
        .create_async()
        .await;

    let client = make_client(&server);
    assert!(client.health_check().await.unwrap());
    mock.assert_async().await;
}

#[tokio::test]
async fn chain_id_returns_correct_value() {
    let mut server = Server::new_async().await;
    let mock = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(rpc_response("0x1"))
        .create_async()
        .await;

    let client = make_client(&server);
    assert_eq!(client.chain_id().await.unwrap(), 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn rpc_error_response_propagated() {
    let mut server = Server::new_async().await;
    let mock = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"execution reverted"}}"#)
        .create_async()
        .await;

    let client = make_client(&server);
    let result = client.chain_id().await;
    assert!(matches!(result, Err(thothterm_web3::error::Web3Error::RpcError { .. })));
    mock.assert_async().await;
}

// ── Project detection ─────────────────────────────────────────────────────────

#[test]
fn detects_foundry_project() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("foundry.toml"), "[profile.default]").unwrap();

    let info = detect_web3_project(dir.path());
    assert!(info);
}

#[test]
fn detects_hardhat_project() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("hardhat.config.ts"), "// hardhat config").unwrap();

    assert!(detect_web3_project(dir.path()));
}

#[test]
fn non_web3_dir_not_detected() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

    assert!(!detect_web3_project(dir.path()));
}

#[test]
fn foundry_project_with_env_rpc() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("foundry.toml"), "[profile.default]").unwrap();
    fs::write(
        dir.path().join(".env"),
        "RPC_URL=https://eth-mainnet.alchemyapi.io/v2/test_key\n"
    ).unwrap();

    use thothterm_web3::detector::detect_project;
    let info = detect_project(dir.path()).unwrap();
    assert_eq!(info.project_type, ProjectType::Foundry);
    assert!(info.rpc_url.is_some());
    assert!(!info.local_node_likely);
}

#[test]
fn foundry_project_with_local_rpc() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("foundry.toml"), "[profile.default]").unwrap();
    fs::write(dir.path().join(".env"), "RPC_URL=http://localhost:8545\n").unwrap();

    use thothterm_web3::detector::detect_project;
    let info = detect_project(dir.path()).unwrap();
    assert!(info.local_node_likely);
}
