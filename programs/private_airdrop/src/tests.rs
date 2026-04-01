//! Tests for the private airdrop program

#[cfg(test)]
mod tests {
    use crate::core::{AirdropAllocation, build_merkle_tree, generate_merkle_proof, verify_merkle_proof};
    
    #[test]
    fn test_complete_airdrop_flow() {
        // Setup phase: distributor creates allocations
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
        ];
        
        // Build Merkle tree and get root
        let leaves: Vec<[u8; 32]> = allocations.iter().map(|a| a.leaf_hash()).collect();
        let (merkle_root, _) = build_merkle_tree(&leaves);
        
        assert_ne!(merkle_root, [0; 32]);
        
        // Recipient 1 generates proof and claims
        let proof = generate_merkle_proof(&leaves, 0).unwrap();
        let allocation = &allocations[0];
        
        // Verify the proof
        let leaf_hash = allocation.leaf_hash();
        assert!(verify_merkle_proof(&leaf_hash, &proof, &merkle_root));
        
        // Recipient 2 generates proof and claims
        let proof2 = generate_merkle_proof(&leaves, 1).unwrap();
        let allocation2 = &allocations[1];
        
        let leaf_hash2 = allocation2.leaf_hash();
        assert!(verify_merkle_proof(&leaf_hash2, &proof2, &merkle_root));
    }
    
    #[test]
    fn test_invalid_proof_rejected() {
        let allocations = vec![
            AirdropAllocation {
                recipient_npk: [1; 32],
                amount: 1000,
                salt: [1; 32],
            },
        ];
        
        let leaves: Vec<[u8; 32]> = allocations.iter().map(|a| a.leaf_hash()).collect();
        let (merkle_root, _) = build_merkle_tree(&leaves);
        
        // Try to use wrong leaf hash
        let wrong_leaf = [99; 32];
        let proof = generate_merkle_proof(&leaves, 0).unwrap();
        
        // This should fail verification
        assert!(!verify_merkle_proof(&wrong_leaf, &proof, &merkle_root));
    }
}
