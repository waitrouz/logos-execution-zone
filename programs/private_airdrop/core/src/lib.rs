//! Private Airdrop Program Core Logic
//! 
//! This module implements the core logic for private airdrops on LEZ.
//! The design uses a Merkle tree commitment scheme where:
//! - Distributor commits to eligibility set via Merkle root
//! - Recipients prove inclusion without revealing their specific position
//! - Nullifiers prevent double-claiming

use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::{
    account::{Account, AccountWithMetadata, Data},
    program::{AccountPostState, Claim},
    Commitment, MembershipProof, compute_digest_for_path,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Airdrop allocation for a single recipient
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct AirdropAllocation {
    /// Recipient's nullifier public key (used for privacy)
    pub recipient_npk: [u8; 32],
    /// Amount of tokens to claim
    pub amount: u128,
    /// Unique salt to ensure unique leaf hashes even for same npk+amount
    pub salt: [u8; 32],
}

impl AirdropAllocation {
    /// Compute the leaf hash for this allocation
    pub fn leaf_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"/LEZ/PrivateAirdrop/v0.1/Leaf\0");
        hasher.update(&self.recipient_npk);
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.salt);
        hasher.finalize().into()
    }
}

/// Airdrop definition account data
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AirdropDefinition {
    /// Token definition ID that this airdrop distributes
    pub token_definition_id: [u8; 32],
    /// Merkle root of eligible recipients
    pub merkle_root: [u8; 32],
    /// Total number of eligible recipients
    pub total_recipients: u64,
    /// Total amount to distribute
    pub total_amount: u128,
    /// Whether the airdrop is active
    pub is_active: bool,
}

impl Default for AirdropDefinition {
    fn default() -> Self {
        Self {
            token_definition_id: [0; 32],
            merkle_root: [0; 32],
            total_recipients: 0,
            total_amount: 0,
            is_active: false,
        }
    }
}

/// Claimant's state - tracks whether they've claimed
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AirdropClaimantState {
    /// Airdrop definition ID
    pub airdrop_definition_id: [u8; 32],
    /// Nullifier to prevent double claiming
    pub nullifier: [u8; 32],
    /// Amount claimed
    pub amount_claimed: u128,
}

/// Input for the claim proof circuit
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ClaimProofInput {
    /// The allocation being claimed
    pub allocation: AirdropAllocation,
    /// Merkle proof of inclusion
    pub membership_proof: MembershipProof,
    /// Expected merkle root
    pub expected_root: [u8; 32],
    /// Nullifier secret key for generating nullifier
    pub nsk: [u8; 32],
}

/// Output from the claim proof circuit
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ClaimProofOutput {
    /// Validity of the proof (true if valid)
    pub is_valid: bool,
    /// Nullifier to prevent double claiming
    pub nullifier: [u8; 32],
    /// The merkle root that was verified against
    pub merkle_root: [u8; 32],
}

/// Compute nullifier from NSK and allocation
pub fn compute_nullifier(nsk: &[u8; 32], allocation: &AirdropAllocation) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"/LEZ/PrivateAirdrop/v0.1/Nullifier\0");
    hasher.update(nsk);
    hasher.update(&allocation.recipient_npk);
    hasher.update(&allocation.salt);
    hasher.finalize().into()
}

/// Verify a Merkle proof
pub fn verify_merkle_proof(
    leaf_hash: &[u8; 32],
    proof: &MembershipProof,
    expected_root: &[u8; 32],
) -> bool {
    // For now, we use a simple verification
    // In production, this would be done in the ZK circuit
    let computed_root = compute_digest_for_path_simple(leaf_hash, proof);
    &computed_root == expected_root
}

/// Simple digest computation for Merkle proofs (host-side)
pub fn compute_digest_for_path_simple(
    leaf_hash: &[u8; 32],
    proof: &MembershipProof,
) -> [u8; 32] {
    let mut result = *leaf_hash;
    let mut level_index = proof.0;
    
    for node in &proof.1 {
        let mut bytes = [0_u8; 64];
        let is_left_child = level_index & 1 == 0;
        if is_left_child {
            bytes[..32].copy_from_slice(&result);
            bytes[32..].copy_from_slice(node);
        } else {
            bytes[..32].copy_from_slice(node);
            bytes[32..].copy_from_slice(&result);
        }
        result = Sha256::digest(&bytes).into();
        level_index >>= 1;
    }
    result
}

/// Build a Merkle tree from leaves and return root and all intermediate nodes
pub fn build_merkle_tree(leaves: &[[u8; 32]]) -> ([u8; 32], Vec<Vec<[u8; 32]>>) {
    if leaves.is_empty() {
        return ([0; 32], vec![]);
    }
    
    let mut current_level: Vec<[u8; 32]> = leaves.to_vec();
    let mut all_levels = vec![current_level.clone()];
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        let mut i = 0;
        while i < current_level.len() {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                left // Duplicate last node if odd number
            };
            
            let mut hasher = Sha256::new();
            hasher.update(&left);
            hasher.update(&right);
            next_level.push(hasher.finalize().into());
            
            i += 2;
        }
        current_level = next_level;
        all_levels.push(current_level.clone());
    }
    
    let root = current_level[0];
    (root, all_levels)
}

/// Generate a Merkle proof for a leaf at given index
pub fn generate_merkle_proof(
    leaves: &[[u8; 32]],
    leaf_index: usize,
) -> Option<MembershipProof> {
    if leaf_index >= leaves.len() {
        return None;
    }
    
    let (_, all_levels) = build_merkle_tree(leaves);
    
    let mut proof = Vec::new();
    let mut index = leaf_index;
    
    for level in all_levels.iter().take(all_levels.len() - 1) {
        let sibling_index = if index % 2 == 0 {
            index + 1
        } else {
            index - 1
        };
        
        let sibling = if sibling_index < level.len() {
            level[sibling_index]
        } else {
            level[index] // Use self if sibling doesn't exist (odd case)
        };
        
        proof.push(sibling);
        index /= 2;
    }
    
    Some((leaf_index, proof))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_leaf_hash() {
        let allocation = AirdropAllocation {
            recipient_npk: [1; 32],
            amount: 1000,
            salt: [2; 32],
        };
        
        let hash = allocation.leaf_hash();
        assert_ne!(hash, [0; 32]);
    }
    
    #[test]
    fn test_merkle_tree() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut leaf = [0; 32];
                leaf[0] = i as u8;
                leaf
            })
            .collect();
        
        let (root, _) = build_merkle_tree(&leaves);
        assert_ne!(root, [0; 32]);
        
        // Test proof generation and verification
        let proof = generate_merkle_proof(&leaves, 1).unwrap();
        let leaf_hash = leaves[1];
        assert!(verify_merkle_proof(&leaf_hash, &proof, &root));
    }
    
    #[test]
    fn test_nullifier_computation() {
        let nsk = [3; 32];
        let allocation = AirdropAllocation {
            recipient_npk: [1; 32],
            amount: 1000,
            salt: [2; 32],
        };
        
        let nullifier = compute_nullifier(&nsk, &allocation);
        assert_ne!(nullifier, [0; 32]);
        
        // Same inputs should produce same nullifier
        let nullifier2 = compute_nullifier(&nsk, &allocation);
        assert_eq!(nullifier, nullifier2);
    }
}
