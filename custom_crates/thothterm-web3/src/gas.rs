use crate::{Web3Client, error::Web3Result, hex_to_u64};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct GasInfo {
    /// Base fee in Gwei.
    pub base_fee_gwei: f64,
    /// Suggested priority fee (tip) in Gwei.
    pub priority_fee_gwei: f64,
    /// Suggested max fee = base_fee * 2 + priority_fee.
    pub max_fee_gwei: f64,
    /// Estimated USD cost for a simple ETH transfer (at $3000/ETH assumed).
    pub simple_transfer_usd: f64,
}

impl GasInfo {
    pub fn display_label(&self) -> String {
        format!("⛽ {:.1}gwei", self.base_fee_gwei)
    }
}

impl Web3Client {
    /// Get current gas prices from the RPC node.
    pub async fn gas_info(&self) -> Web3Result<GasInfo> {
        // eth_gasPrice returns current gas price in Wei
        let gas_price_hex: String = self
            .call("eth_gasPrice", serde_json::json!([]))
            .await?;

        let gas_price_wei = hex_to_u64(&gas_price_hex)?;
        let base_fee_gwei = gas_price_wei as f64 / 1e9;

        // Typical priority fee is 1-2 Gwei
        let priority_fee_gwei = 1.5;
        let max_fee_gwei = base_fee_gwei * 1.2 + priority_fee_gwei;

        // Simple ETH transfer uses 21,000 gas
        let simple_transfer_eth = max_fee_gwei * 21_000.0 / 1e9;
        let simple_transfer_usd = simple_transfer_eth * 3000.0; // approximate

        debug!("Gas price: {:.2} Gwei", base_fee_gwei);

        Ok(GasInfo {
            base_fee_gwei,
            priority_fee_gwei,
            max_fee_gwei,
            simple_transfer_usd,
        })
    }
}
