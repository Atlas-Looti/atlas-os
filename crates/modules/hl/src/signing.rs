//! EIP-712 Agent signing for Hyperliquid exchange actions.
//!
//! Used for action types not exposed in hypersdk's Action enum
//! (e.g. updateLeverage).

use alloy::primitives::{keccak256, B256};

/// Compute the EIP-712 signing hash for a Hyperliquid Agent action.
///
/// Domain: name="Exchange", version="1", chainId=1337, verifyingContract=0x0
pub fn compute_agent_signing_hash(source: &str, connection_id: B256) -> B256 {
    let domain_type_hash = keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );

    let mut domain_data = Vec::with_capacity(160);
    domain_data.extend_from_slice(domain_type_hash.as_slice());
    domain_data.extend_from_slice(keccak256(b"Exchange").as_slice());
    domain_data.extend_from_slice(keccak256(b"1").as_slice());
    let mut chain_id_bytes = [0u8; 32];
    chain_id_bytes[31] = (1337 & 0xFF) as u8;
    chain_id_bytes[30] = ((1337 >> 8) & 0xFF) as u8;
    domain_data.extend_from_slice(&chain_id_bytes);
    domain_data.extend_from_slice(&[0u8; 32]);

    let domain_separator = keccak256(&domain_data);

    let agent_type_hash = keccak256(b"Agent(string source,bytes32 connectionId)");

    let mut struct_data = Vec::with_capacity(96);
    struct_data.extend_from_slice(agent_type_hash.as_slice());
    struct_data.extend_from_slice(keccak256(source.as_bytes()).as_slice());
    struct_data.extend_from_slice(connection_id.as_slice());

    let struct_hash = keccak256(&struct_data);

    let mut final_data = Vec::with_capacity(66);
    final_data.push(0x19);
    final_data.push(0x01);
    final_data.extend_from_slice(domain_separator.as_slice());
    final_data.extend_from_slice(struct_hash.as_slice());

    keccak256(&final_data)
}
