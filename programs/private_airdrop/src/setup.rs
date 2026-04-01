//! Setup utilities for the private airdrop program

use private_airdrop_core::{AirdropAllocation, build_merkle_tree, generate_merkle_proof};

/// Helper struct for setting up an airdrop distribution
pub struct AirdropSetup {
    allocations: Vec<AirdropAllocation>,
    merkle_root: [u8; 32],
}

impl AirdropSetup {
    /// Create a new airdrop setup from allocations
    pub fn new(allocations: Vec<AirdropAllocation>) -> Self {
        let leaves: Vec<[u8; 32]> = allocations.iter().map(|a| a.leaf_hash()).collect();
        let (merkle_root, _) = build_merkle_tree(&leaves);
        
        Self {
            allocations,
            merkle_root,
        }
    }
    
    /// Get the merkle root
    pub fn merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root
    }
    
    /// Get total recipients
    pub fn total_recipients(&self) -> u64 {
        self.allocations.len() as u64
    }
    
    /// Get total amount
    pub fn total_amount(&self) -> u128 {
        self.allocations.iter().map(|a| a.amount).sum()
    }
    
    /// Generate a claim package for a specific recipient
    /// Returns the allocation and merkle proof for the recipient at given index
    pub fn generate_claim_package(
        &self,
        recipient_index: usize,
    ) -> Option<(AirdropAllocation, private_airdrop_core::MembershipProof)> {
        if recipient_index >= self.allocations.len() {
            return None;
        }
        
        let allocation = self.allocations[recipient_index].clone();
        let leaves: Vec<[u8; 32]> = self.allocations.iter().map(|a| a.leaf_hash()).collect();
        let proof = generate_merkle_proof(&leaves, recipient_index)?;
        
        Some((allocation, proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_airdrop_setup() {
        let allocations = vec![
            AirdropAllocation {
                recipient_npk: [1; 32],
                amount: 1000,
                salt: [1; 32],
            },
            AirdropAllocation {
                recipient_npk: [2; 32],
                amount: 2000,
                salt: [2; 32],
            },
            AirdropAllocation {
                recipient_npk: [3; 32],
                amount: 3000,
                salt: [3; 32],
            },
        ];
        
        let setup = AirdropSetup::new(allocations.clone());
        
        assert_eq!(setup.total_recipients(), 3);
        assert_eq!(setup.total_amount(), 6000);
        assert_ne!(*setup.merkle_root(), [0; 32]);
        
        // Test claim package generation
        let (allocation, proof) = setup.generate_claim_package(1).unwrap();
        assert_eq!(allocation.amount, 2000);
        assert!(!proof.1.is_empty());
    }
}
