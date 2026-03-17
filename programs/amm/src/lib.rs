//! The AMM Program implementation.

pub use amm_core as core;

pub mod add;
pub mod new_definition;
pub mod remove;
pub mod swap;

#[cfg(test)]
mod tests;
