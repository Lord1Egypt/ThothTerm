use crate::error::{Web3Error, Web3Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A minimal Ethereum wallet (address + encrypted key storage).
/// Uses alloy-primitives for address parsing when the `ethereum` feature is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Checksummed Ethereum address ("0x...")
    pub address: String,
    /// Optional human-readable label.
    pub label: Option<String>,
}

impl Wallet {
    /// Create a wallet entry from a raw address (no key import — read-only).
    pub fn from_address(address: impl Into<String>, label: Option<String>) -> Web3Result<Self> {
        let address = address.into();
        crate::validate_address(&address)?;
        Ok(Self { address, label })
    }

    /// Return a shortened address for display: "0x1234…abcd".
    pub fn display_address(&self) -> String {
        if self.address.len() < 10 {
            return self.address.clone();
        }
        format!("{}…{}", &self.address[..6], &self.address[self.address.len() - 4..])
    }
}

/// Simple wallet store backed by a JSON file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WalletStore {
    pub wallets: Vec<Wallet>,
    pub active_index: Option<usize>,
}

impl WalletStore {
    /// Load from a JSON file, returning an empty store if the file doesn't exist.
    pub fn load(path: &Path) -> Web3Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path).map_err(|e| Web3Error::IoError(e.to_string()))?;
        serde_json::from_str(&data).map_err(|e| Web3Error::ParseError(e.into()))
    }

    /// Save to a JSON file.
    pub fn save(&self, path: &Path) -> Web3Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Web3Error::IoError(e.to_string()))?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data).map_err(|e| Web3Error::IoError(e.to_string()))
    }

    /// Add a wallet, returning its index.
    pub fn add(&mut self, wallet: Wallet) -> usize {
        self.wallets.push(wallet);
        self.wallets.len() - 1
    }

    /// Get the active wallet.
    pub fn active(&self) -> Option<&Wallet> {
        self.active_index.and_then(|i| self.wallets.get(i))
    }

    /// Set active wallet by index.
    pub fn set_active(&mut self, index: usize) -> Web3Result<()> {
        if index >= self.wallets.len() {
            return Err(Web3Error::InvalidAddress(format!("wallet index {} out of range", index)));
        }
        self.active_index = Some(index);
        Ok(())
    }

    /// Default wallet store path, respects portable mode.
    pub fn default_path() -> std::path::PathBuf {
        thothterm_config::RunMode::detect().wallet_store_path()
    }
}
