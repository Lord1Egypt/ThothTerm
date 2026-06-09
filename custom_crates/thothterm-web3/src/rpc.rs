use crate::Web3Client;
use std::time::Instant;

/// Result of an RPC health check.
#[derive(Debug, Clone)]
pub struct RpcHealthInfo {
    pub url: String,
    pub is_healthy: bool,
    pub chain_id: Option<u64>,
    pub block_number: Option<u64>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

impl RpcHealthInfo {
    pub fn status_label(&self) -> &'static str {
        if self.is_healthy {
            "🟢"
        } else {
            "🔴"
        }
    }
}

impl Web3Client {
    /// Full health check — returns chain ID, block number, and latency.
    pub async fn full_health_check(&self) -> RpcHealthInfo {
        let url = self.rpc_url.clone();
        let start = Instant::now();

        match self.chain_id().await {
            Ok(chain_id) => {
                let block = self.block_number().await.ok();
                RpcHealthInfo {
                    url,
                    is_healthy: true,
                    chain_id: Some(chain_id),
                    block_number: block,
                    latency_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            Err(e) => RpcHealthInfo {
                url,
                is_healthy: false,
                chain_id: None,
                block_number: None,
                latency_ms: start.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            },
        }
    }
}

/// Get a human-readable chain name from chain ID.
pub fn chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "Ethereum Mainnet",
        5 => "Goerli Testnet",
        11155111 => "Sepolia Testnet",
        137 => "Polygon",
        80001 => "Polygon Mumbai",
        42161 => "Arbitrum One",
        10 => "Optimism",
        8453 => "Base",
        56 => "BNB Chain",
        43114 => "Avalanche C-Chain",
        31337 => "Anvil/Hardhat Local",
        1337 => "Ganache Local",
        _ => "Unknown Network",
    }
}
