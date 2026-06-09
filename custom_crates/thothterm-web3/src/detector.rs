use std::path::Path;

/// Describes what kind of Web3 project is in a directory.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Foundry,
    Hardhat,
    Truffle,
    Brownie,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Web3ProjectInfo {
    pub project_type: ProjectType,
    /// RPC URL from .env file, if found.
    pub rpc_url: Option<String>,
    /// Whether Anvil/Ganache is likely running locally.
    pub local_node_likely: bool,
}

/// Detect if a directory is a Web3 project and extract useful info.
pub fn detect_project(dir: &Path) -> Option<Web3ProjectInfo> {
    let project_type = if dir.join("foundry.toml").exists() {
        ProjectType::Foundry
    } else if dir.join("hardhat.config.js").exists() || dir.join("hardhat.config.ts").exists() {
        ProjectType::Hardhat
    } else if dir.join("truffle-config.js").exists() {
        ProjectType::Truffle
    } else if dir.join("brownie-config.yaml").exists() {
        ProjectType::Brownie
    } else {
        return None;
    };

    // Try to read RPC URL from .env
    let rpc_url = read_rpc_from_env(dir);

    // Check if a local node might be running
    let local_node_likely = rpc_url
        .as_deref()
        .map(|url| url.contains("localhost") || url.contains("127.0.0.1"))
        .unwrap_or(false);

    Some(Web3ProjectInfo {
        project_type,
        rpc_url,
        local_node_likely,
    })
}

fn read_rpc_from_env(dir: &Path) -> Option<String> {
    let env_path = dir.join(".env");
    if !env_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&env_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Look for common RPC URL variable names
        for prefix in &["RPC_URL=", "ETH_RPC_URL=", "MAINNET_RPC_URL=", "ALCHEMY_RPC="] {
            if let Some(val) = line.strip_prefix(prefix) {
                let url = val.trim_matches('"').trim_matches('\'').trim().to_string();
                if !url.is_empty() {
                    return Some(url);
                }
            }
        }
    }

    None
}
