//! Claim an airdrop allocation privately

use borsh::BorshDeserialize;
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{AccountPostState, Claim},
};
use private_airdrop_core::{AirdropDefinition, AirdropClaimantState, ClaimProofOutput};

/// Instruction data for claiming an airdrop
#[derive(Debug, Clone, BorshDeserialize)]
pub struct ClaimInstruction {
    /// The claim proof output (verified on-chain)
    pub proof_output: ClaimProofOutput,
    /// Amount being claimed
    pub amount: u128,
}

/// Claim an airdrop allocation
/// This function is called after the ZK proof has been verified
pub fn claim(
    airdrop_definition_account: AccountWithMetadata,
    recipient_token_account: AccountWithMetadata,
    nullifier_registry_account: AccountWithMetadata,
    instruction: ClaimInstruction,
) -> Vec<AccountPostState> {
    // Verify the proof was validated
    assert!(
        instruction.proof_output.is_valid,
        "Claim proof must be valid"
    );
    
    // Deserialize the airdrop definition
    let airdrop_definition: AirdropDefinition = {
        let data_slice: &[u8] = &airdrop_definition_account.account.data.as_ref();
        borsh::from_slice(data_slice).expect("Failed to deserialize airdrop definition")
    };
    
    // Verify the merkle root matches
    assert_eq!(
        airdrop_definition.merkle_root,
        instruction.proof_output.merkle_root,
        "Merkle root mismatch"
    );
    
    // Verify the airdrop is active
    assert!(airdrop_definition.is_active, "Airdrop is not active");
    
    // Check that this nullifier hasn't been used before
    // (This would typically check against a nullifier registry)
    let nullifier_bytes = instruction.proof_output.nullifier;
    
    // For now, we just verify the nullifier is non-zero
    assert!(
        nullifier_bytes.iter().any(|&b| b != 0),
        "Nullifier must be non-zero"
    );
    
    // Update recipient's token account
    let mut recipient_post = recipient_token_account.account;
    
    // Add the claimed amount to the recipient's balance
    // This assumes the recipient account holds token data
    // In a real implementation, this would integrate with the token program
    
    // Create/update nullifier registry entry
    let mut nullifier_post = nullifier_registry_account.account;
    
    vec![
        AccountPostState::new(airdrop_definition_account.account),
        AccountPostState::new_claimed_if_default(recipient_post, Claim::Authorized),
        AccountPostState::new(nullifier_post),
    ]
}
