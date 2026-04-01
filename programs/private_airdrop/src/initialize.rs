//! Initialize a new airdrop definition

use borsh::BorshDeserialize;
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{AccountPostState, Claim},
};
use private_airdrop_core::AirdropDefinition;

/// Instruction data for initializing an airdrop
#[derive(Debug, Clone, BorshDeserialize)]
pub struct InitializeInstruction {
    /// Token definition ID that this airdrop distributes
    pub token_definition_id: [u8; 32],
    /// Merkle root of eligible recipients
    pub merkle_root: [u8; 32],
    /// Total number of eligible recipients
    pub total_recipients: u64,
    /// Total amount to distribute
    pub total_amount: u128,
}

/// Initialize a new airdrop definition account
pub fn initialize(
    airdrop_definition_account: AccountWithMetadata,
    distributor_account: AccountWithMetadata,
    instruction: InitializeInstruction,
) -> Vec<AccountPostState> {
    assert!(
        distributor_account.is_authorized,
        "Distributor must be authorized"
    );
    
    // Verify the airdrop definition account is unclaimed (default state)
    assert_eq!(
        airdrop_definition_account.account,
        Account::default(),
        "Airdrop definition account must be uninitialized"
    );
    
    // Create the airdrop definition
    let airdrop_definition = AirdropDefinition {
        token_definition_id: instruction.token_definition_id,
        merkle_root: instruction.merkle_root,
        total_recipients: instruction.total_recipients,
        total_amount: instruction.total_amount,
        is_active: true,
    };
    
    // Serialize the definition into account data
    let mut data = Vec::new();
    borsh::to_writer(&mut data, &airdrop_definition)
        .expect("Failed to serialize airdrop definition");
    
    // Pad or truncate to fit account data size
    let mut account_data = [0u8; 256]; // Adjust based on actual account data size
    let copy_len = data.len().min(account_data.len());
    account_data[..copy_len].copy_from_slice(&data[..copy_len]);
    
    let mut post_account = airdrop_definition_account.account;
    post_account.data = nssa_core::account::Data::from(account_data);
    
    vec![
        AccountPostState::new(post_account),
        AccountPostState::new(distributor_account.account),
    ]
}
