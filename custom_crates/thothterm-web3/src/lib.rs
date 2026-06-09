pub mod error;
pub mod rpc;
pub mod gas;
pub mod ens;
pub mod detector;
pub mod wallet;

use error::{Web3Error, Web3Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

const RPC_TIMEOUT_SECS: u64 = 10;

// ── JSON-RPC types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct RpcRequest {
    pub jsonrpc: &'static str,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<RpcResponseError>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RpcResponseError {
    pub code: i64,
    pub message: String,
}

// ── Web3 client ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Web3Client {
    rpc_url: String,
    http: Client,
}

impl Web3Client {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(RPC_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            rpc_url: rpc_url.into(),
            http,
        }
    }

    pub fn from_config(config: &thothterm_config::Web3Config) -> Web3Result<Self> {
        if !config.enabled {
            return Err(Web3Error::Disabled);
        }
        if config.rpc_url.is_empty() {
            return Err(Web3Error::NoRpcUrl);
        }
        Ok(Self::new(&config.rpc_url))
    }

    /// Send a raw JSON-RPC request.
    pub(crate) async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Web3Result<T> {
        let body = RpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: 1,
        };

        debug!("RPC call: {}", method);

        let resp = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?;

        let rpc_resp: RpcResponse<T> = resp.json().await?;

        if let Some(err) = rpc_resp.error {
            return Err(Web3Error::RpcError {
                code: err.code,
                message: err.message,
            });
        }

        rpc_resp.result.ok_or_else(|| Web3Error::RpcError {
            code: -1,
            message: "Empty result".into(),
        })
    }

    /// Check if the RPC endpoint is healthy.
    pub async fn health_check(&self) -> Web3Result<bool> {
        let _block_number: String = self.call("eth_blockNumber", serde_json::json!([])).await?;
        Ok(true)
    }

    /// Get the current network chain ID.
    pub async fn chain_id(&self) -> Web3Result<u64> {
        let hex: String = self.call("eth_chainId", serde_json::json!([])).await?;
        hex_to_u64(&hex)
    }

    /// Get ETH balance for an address (in Wei, as hex string).
    pub async fn get_balance(&self, address: &str) -> Web3Result<u64> {
        validate_address(address)?;
        let hex: String = self
            .call("eth_getBalance", serde_json::json!([address, "latest"]))
            .await?;
        hex_to_u64(&hex)
    }

    /// Get current block number.
    pub async fn block_number(&self) -> Web3Result<u64> {
        let hex: String = self.call("eth_blockNumber", serde_json::json!([])).await?;
        hex_to_u64(&hex)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn validate_address(addr: &str) -> Web3Result<()> {
    if !addr.starts_with("0x") || addr.len() != 42 {
        return Err(Web3Error::InvalidAddress(addr.to_string()));
    }
    if !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Web3Error::InvalidAddress(addr.to_string()));
    }
    Ok(())
}

pub fn hex_to_u64(hex: &str) -> Web3Result<u64> {
    let stripped = hex.strip_prefix("0x").unwrap_or(hex);
    u64::from_str_radix(stripped, 16).map_err(|_| Web3Error::ParseError(
        serde_json::from_str::<()>("").unwrap_err()
    ))
}

/// Format Wei as ETH string (e.g., "1.234 ETH").
pub fn wei_to_eth_string(wei: u64) -> String {
    let eth = wei as f64 / 1e18;
    format!("{:.4} ETH", eth)
}

/// Check if an address looks like an ENS name (ends with .eth).
pub fn is_ens_name(s: &str) -> bool {
    s.ends_with(".eth") && s.len() > 4
}

/// Check if a directory looks like an Ethereum/Web3 project.
pub fn detect_web3_project(dir: &std::path::Path) -> bool {
    let markers = [
        "foundry.toml",
        "hardhat.config.js",
        "hardhat.config.ts",
        "truffle-config.js",
        "brownie-config.yaml",
    ];
    markers.iter().any(|m| dir.join(m).exists())
}
