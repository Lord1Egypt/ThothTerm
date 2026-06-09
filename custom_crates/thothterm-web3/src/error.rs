use thiserror::Error;

#[derive(Debug, Error)]
pub enum Web3Error {
    #[error("Web3 is disabled in config")]
    Disabled,

    #[error("No RPC URL configured")]
    NoRpcUrl,

    #[error("RPC request failed: {0}")]
    RpcFailed(#[from] reqwest::Error),

    #[error("RPC server returned error: {code} — {message}")]
    RpcError { code: i64, message: String },

    #[error("Invalid Ethereum address: {0}")]
    InvalidAddress(String),

    #[error("ENS resolution failed for {name}: {reason}")]
    EnsResolutionFailed { name: String, reason: String },

    #[error("Failed to parse RPC response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Network not supported: {0}")]
    UnsupportedNetwork(String),

    #[error("Request timed out")]
    Timeout,

    #[error("I/O error: {0}")]
    IoError(String),
}

pub type Web3Result<T> = Result<T, Web3Error>;
