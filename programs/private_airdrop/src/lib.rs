//! Private Airdrop Program for LEZ
//! 
//! This program implements a privacy-preserving airdrop mechanism where:
//! - Distributors can commit to an eligibility set without revealing addresses
//! - Recipients can claim their allocation privately
//! - Double-claiming is prevented via nullifiers
//! - On-chain observers cannot link claims to specific eligible addresses

pub use private_airdrop_core as core;

pub mod initialize;
pub mod claim;
pub mod setup;

mod tests;
