//! ZK Circuit for Private Airdrop Claim Proof
//! 
//! This circuit proves:
//! 1. The claimant knows a leaf (their address + secret) in the Merkle tree
//! 2. The leaf corresponds to a valid allocation
//! 3. The nullifier is correctly computed to prevent double-claims
//! 
//! The proof reveals NOTHING about which leaf was used or the claimant's identity.

#![no_main]
use risc0_zkvm::guest::env;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Merkle proof structure
#[derive(Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub siblings: Vec<[u8; 32]>,
}

/// Public inputs to the circuit (visible on-chain)
#[derive(Serialize, Deserialize)]
pub struct PublicInputs {
    /// Merkle root of the eligibility tree
    pub merkle_root: [u8; 32],
    /// Nullifier to prevent double-claims
    pub nullifier: [u8; 32],
    /// Commitment to the claimed amount (optional privacy)
    pub amount_commitment: [u8; 32],
}

/// Private inputs (witness) - NEVER revealed
#[derive(Serialize, Deserialize)]
pub struct PrivateInputs {
    /// Recipient's shielded address
    pub address: [u8; 32],
    /// Secret used for nullifier computation
    pub nullifier_secret: [u8; 32],
    /// Allocation amount for this recipient
    pub amount: u64,
    /// Merkle proof of inclusion
    pub merkle_proof: MerkleProof,
}

/// Compute SHA256 hash
fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute Merkle root from leaf and proof
fn compute_merkle_root(leaf: [u8; 32], proof: &MerkleProof) -> [u8; 32] {
    let mut current_hash = leaf;
    
    for (i, sibling) in proof.siblings.iter().enumerate() {
        let mut data = Vec::with_capacity(64);
        
        // Determine order based on bit at position i of leaf_index
        if (proof.leaf_index >> i) & 1 == 0 {
            data.extend_from_slice(&current_hash);
            data.extend_from_slice(sibling);
        } else {
            data.extend_from_slice(sibling);
            data.extend_from_slice(&current_hash);
        }
        
        current_hash = hash_sha256(&data);
    }
    
    current_hash
}

fn main() {
    // Read public inputs
    let public_inputs: PublicInputs = env::read();
    
    // Read private inputs (witness)
    let private_inputs: PrivateInputs = env::read();
    
    // === STEP 1: Verify Merkle proof ===
    // Construct the leaf: hash(address || amount || nullifier_secret)
    let mut leaf_data = Vec::with_capacity(96);
    leaf_data.extend_from_slice(&private_inputs.address);
    leaf_data.extend_from_slice(&private_inputs.amount.to_le_bytes());
    leaf_data.extend_from_slice(&private_inputs.nullifier_secret);
    let leaf = hash_sha256(&leaf_data);
    
    // Compute expected root from proof
    let computed_root = compute_merkle_root(leaf, &private_inputs.merkle_proof);
    
    // Verify root matches
    env::verify(computed_root == public_inputs.merkle_root).expect("Merkle proof verification failed");
    
    // === STEP 2: Verify nullifier computation ===
    // Nullifier = hash(nullifier_secret || address)
    let mut nullifier_data = Vec::with_capacity(64);
    nullifier_data.extend_from_slice(&private_inputs.nullifier_secret);
    nullifier_data.extend_from_slice(&private_inputs.address);
    let computed_nullifier = hash_sha256(&nullifier_data);
    
    env::verify(computed_nullifier == public_inputs.nullifier)
        .expect("Nullifier computation mismatch");
    
    // === STEP 3: Verify amount commitment ===
    // Commitment = hash(amount || random_blinder)
    // For simplicity, we use nullifier_secret as blinder (in production, use separate random)
    let mut commitment_data = Vec::with_capacity(40);
    commitment_data.extend_from_slice(&private_inputs.amount.to_le_bytes());
    commitment_data.extend_from_slice(&private_inputs.nullifier_secret[..8]);
    let computed_commitment = hash_sha256(&commitment_data);
    
    env::verify(computed_commitment == public_inputs.amount_commitment)
        .expect("Amount commitment mismatch");
    
    // === STEP 4: Range check on amount ===
    // Ensure amount is positive and reasonable (e.g., < 2^53)
    env::verify(private_inputs.amount > 0).expect("Amount must be positive");
    env::verify(private_inputs.amount < (1u64 << 53)).expect("Amount too large");
    
    // If all checks pass, the proof is valid
    env::commit(&public_inputs);
}
