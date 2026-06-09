use crate::{
    error::{Web3Error, Web3Result},
    Web3Client,
};
use tiny_keccak::{Hasher, Keccak};

// ENS registry on Ethereum mainnet
const ENS_REGISTRY: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";

// keccak256(bytes) → [u8; 32]
fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(data);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

/// ENS namehash algorithm (EIP-137).
/// namehash("") = 0x000...000
/// namehash("foo.eth") = keccak256(namehash("eth") + keccak256("foo"))
pub fn namehash(name: &str) -> [u8; 32] {
    let mut node = [0u8; 32];
    if name.is_empty() {
        return node;
    }
    let labels: Vec<&str> = name.split('.').collect();
    for label in labels.iter().rev() {
        let label_hash = keccak256(label.as_bytes());
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&node);
        combined[32..].copy_from_slice(&label_hash);
        node = keccak256(&combined);
    }
    node
}

/// Encode a call to `resolver(bytes32 node)` on the ENS registry.
fn encode_resolver_call(node: &[u8; 32]) -> String {
    // Function selector: keccak256("resolver(bytes32)")[..4] = 0x0178b8bf
    let selector = "0178b8bf";
    let node_hex: String = node.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}{}", selector, node_hex)
}

/// Encode a call to `addr(bytes32 node)` on a resolver.
fn encode_addr_call(node: &[u8; 32]) -> String {
    // Function selector: keccak256("addr(bytes32)")[..4] = 0x3b3b57de
    let selector = "3b3b57de";
    let node_hex: String = node.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}{}", selector, node_hex)
}

/// Extract a 20-byte Ethereum address from a 32-byte ABI-padded hex result.
fn decode_address(hex_result: &str) -> Option<String> {
    let stripped = hex_result.strip_prefix("0x").unwrap_or(hex_result);
    if stripped.len() < 40 {
        return None;
    }
    let addr = format!("0x{}", &stripped[stripped.len() - 40..]);
    if addr == "0x0000000000000000000000000000000000000000" {
        return None;
    }
    Some(addr)
}

impl Web3Client {
    /// Resolve an ENS name to an Ethereum address using the ENS registry.
    /// Flow: registry.resolver(node) → resolver.addr(node)
    pub async fn resolve_ens(&self, name: &str) -> Web3Result<String> {
        let name = name.to_lowercase();
        if !name.ends_with(".eth") && !name.contains('.') {
            return Err(Web3Error::EnsResolutionFailed {
                name: name.clone(),
                reason: "ENS names must end with .eth or contain a dot".into(),
            });
        }

        let node = namehash(&name);

        // Step 1: get the resolver address from the registry
        let resolver_call = encode_resolver_call(&node);
        let resolver_hex: String = self
            .call(
                "eth_call",
                serde_json::json!([
                    { "to": ENS_REGISTRY, "data": resolver_call },
                    "latest"
                ]),
            )
            .await
            .map_err(|e| Web3Error::EnsResolutionFailed {
                name: name.clone(),
                reason: format!("registry lookup failed: {}", e),
            })?;

        let resolver_addr = decode_address(&resolver_hex).ok_or_else(|| {
            Web3Error::EnsResolutionFailed {
                name: name.clone(),
                reason: "no resolver set for this name".into(),
            }
        })?;

        // Step 2: call addr(node) on the resolver
        let addr_call = encode_addr_call(&node);
        let addr_hex: String = self
            .call(
                "eth_call",
                serde_json::json!([
                    { "to": resolver_addr, "data": addr_call },
                    "latest"
                ]),
            )
            .await
            .map_err(|e| Web3Error::EnsResolutionFailed {
                name: name.clone(),
                reason: format!("resolver addr() call failed: {}", e),
            })?;

        decode_address(&addr_hex).ok_or_else(|| Web3Error::EnsResolutionFailed {
            name: name.clone(),
            reason: "name resolves to zero address (not registered)".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namehash_eth() {
        let node = namehash("eth");
        // keccak256(0x00..00 + keccak256("eth")) — known value
        let expected = "93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae";
        let hex: String = node.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, expected);
    }

    #[test]
    fn test_namehash_empty() {
        let node = namehash("");
        assert_eq!(node, [0u8; 32]);
    }

    #[test]
    fn test_decode_address() {
        let padded = "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045";
        let addr = decode_address(padded).unwrap();
        assert_eq!(addr, "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
    }
}
