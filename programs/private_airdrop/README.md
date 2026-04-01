# Private Airdrop Program for LEZ

A privacy-preserving airdrop and allowlist distribution primitive for Logos Execution Zone (LEZ).

## Overview

This program enables:
- **Distributors** to commit to an eligibility set on-chain without revealing individual addresses
- **Recipients** to claim their allocation without revealing which address in the set they hold
- **Double-claim prevention** via nullifiers
- **Privacy** where on-chain observers cannot link a completed claim to any specific eligible address

## Privacy Model

### What On-Chain Observers Learn
- The Merkle root of the eligible set (committed at setup)
- Total number of eligible recipients
- Total amount to distribute
- That *someone* claimed (but not who)
- The nullifier (prevents double-claiming but doesn't reveal identity)

### What the Distributor Learns
- At setup: Complete knowledge of all eligible addresses and amounts (required to build Merkle tree)
- At claim time: Nothing - claims are submitted anonymously with ZK proofs

### When Identity Information is Revealed/Withheld
1. **Setup Phase**: Distributor knows all eligible addresses (unavoidable - they must know who to include)
2. **Commitment Phase**: Only Merkle root published on-chain (individual addresses hidden)
3. **Claim Phase**: Recipient proves inclusion via ZK proof without revealing their position in the tree
4. **Post-Claim**: Nullifier prevents re-claiming but doesn't link to original address

### Threat Model
- **Assumptions**: 
  - Risc0 proving system is sound (cannot forge valid proofs)
  - Hash functions (SHA-256) are collision-resistant
  - Nullifier derivation is deterministic and one-way
  
- **Guarantees**:
  - Unlinkability: Claims cannot be linked to specific eligible addresses by on-chain observers
  - One-time claim: Each eligible recipient can claim exactly once
  - Validity: Only eligible recipients can claim (proven via Merkle inclusion)

- **Residual Leakage/Limitations**:
  - Distributor knows the full eligibility list at setup (by design)
  - Total number of recipients and total amount are public
  - Timing analysis might correlate claim times with distributor communications
  - Does not hide the fact that a claim occurred, only who claimed

## Architecture

### Components

1. **Core Library** (`programs/private_airdrop/core/`)
   - `AirdropAllocation`: Represents a single recipient's allocation
   - `AirdropDefinition`: On-chain account storing airdrop metadata
   - `AirdropClaimantState`: Tracks claim state per recipient
   - Merkle tree utilities for commitment and proof generation
   - Nullifier computation for double-claim prevention

2. **Program Modules** (`programs/private_airdrop/src/`)
   - `initialize`: Creates new airdrop definition accounts
   - `setup`: Helper utilities for distributors to prepare airdrops
   - `claim`: Processes private claims with proof verification
   - `tests`: Integration tests

### Commitment Scheme

Uses a Merkle tree where:
- **Leaves**: Hash of `(recipient_npk, amount, salt)` using domain-separated SHA-256
- **Root**: Published on-chain as commitment to eligibility set
- **Proofs**: Standard Merkle inclusion proofs verified in ZK circuit

### Claim Uniqueness Mechanism

Nullifiers prevent double-claiming:
```
nullifier = SHA256(domain || nsk || recipient_npk || salt)
```

Where:
- `nsk`: Recipient's nullifier secret key (never revealed)
- `recipient_npk`: Nullifier public key (in leaf hash)
- `salt`: Unique per-allocation to ensure unique nullifiers

The nullifier is published on-chain when claiming. Subsequent claims with the same nullifier are rejected.

## Installation

### Prerequisites

```bash
# Install build dependencies (Ubuntu/Debian)
apt install build-essential clang libclang-dev libssl-dev pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Risc0
curl -L https://risczero.com/install | bash
rzup install
```

### Build

```bash
cd /workspace

# Build the private airdrop program
cargo build --release -p private_airdrop

# Build the core library
cargo build --release -p private_airdrop_core

# Run unit tests
RISC0_DEV_MODE=1 cargo test --release -p private_airdrop -p private_airdrop_core
```

## Usage

### For Distributors: Setting Up an Airdrop

```rust
use private_airdrop_core::{AirdropAllocation, build_merkle_tree};
use private_airdrop::setup::AirdropSetup;

// 1. Define allocations
let allocations = vec![
    AirdropAllocation {
        recipient_npk: [/* recipient 1 npk */],
        amount: 1000,
        salt: [/* random salt */],
    },
    AirdropAllocation {
        recipient_npk: [/* recipient 2 npk */],
        amount: 2000,
        salt: [/* random salt */],
    },
    // ... more recipients
];

// 2. Create setup helper
let setup = AirdropSetup::new(allocations);

// 3. Get commitment data
let merkle_root = setup.merkle_root();
let total_recipients = setup.total_recipients();
let total_amount = setup.total_amount();

// 4. Initialize on-chain airdrop definition
// (See initialize.rs for transaction construction)
```

### For Recipients: Claiming an Airdrop

```rust
use private_airdrop_core::{verify_merkle_proof, compute_nullifier};
use private_airdrop::setup::AirdropSetup;

// 1. Receive claim package from distributor (off-chain)
//    Contains: allocation, merkle_proof

// 2. Generate ZK proof (client-side)
//    Proves: 
//    - Knowledge of allocation with valid Merkle proof
//    - Correct nullifier derivation
//    - Without revealing which leaf in the tree

// 3. Submit claim transaction with proof
// (See claim.rs for transaction construction)
```

### Generating Claim Packages

```rust
// Distributor generates claim package for recipient at index 5
let (allocation, merkle_proof) = setup.generate_claim_package(5)?;

// Send to recipient off-chain (encrypted channel recommended)
// Recipient uses this to generate their claim proof
```

## Testing

### Unit Tests

```bash
# Run core library tests
RISC0_DEV_MODE=1 cargo test --release -p private_airdrop_core

# Run program tests
RISC0_DEV_MODE=1 cargo test --release -p private_airdrop
```

### Integration Test

Create `integration_tests/tests/private_airdrop.rs`:

```rust
#![expect(
    clippy::shadow_unrelated,
    clippy::tests_outside_test_module,
    reason = "We don't care about these in tests"
)]

use std::time::Duration;
use anyhow::{Context as _, Result};
use integration_tests::{TIME_TO_WAIT_FOR_BLOCK_SECONDS, TestContext};
use log::info;
use tokio::test;
use private_airdrop_core::{AirdropAllocation, AirdropSetup};

#[test]
async fn create_and_claim_private_airdrop() -> Result<()> {
    let mut ctx = TestContext::new().await?;

    // Setup: Create airdrop with 3 recipients
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

    let setup = AirdropSetup::new(allocations);
    let merkle_root = *setup.merkle_root();
    
    info!("Merkle root: {:?}", hex::encode(merkle_root));
    info!("Total recipients: {}", setup.total_recipients());
    info!("Total amount: {}", setup.total_amount());

    // TODO: Implement full integration test with:
    // 1. Initialize airdrop definition account
    // 2. Generate claim packages for recipients
    // 3. Recipients generate ZK proofs
    // 4. Submit claims and verify on-chain state
    
    Ok(())
}
```

Run integration tests:

```bash
export NSSA_WALLET_HOME_DIR=$(pwd)/integration_tests/configs/debug/wallet/
cd integration_tests
RUST_LOG=info RISC0_DEV_MODE=1 cargo run $(pwd)/configs/debug all
```

## Deployment to LEZ Testnet

### Step 1: Build Program

```bash
cargo build --release -p private_airdrop
```

### Step 2: Deploy Program

```bash
# Use LEZ CLI to deploy
lez-cli program deploy \
  --path target/release/libprivate_airdrop.so \
  --network testnet
```

### Step 3: Create Airdrop Distribution

```bash
# Prepare allocation file (JSON)
cat > allocations.json << 'EOF'
[
  {"recipient_npk": "...", "amount": 1000, "salt": "..."},
  {"recipient_npk": "...", "amount": 2000, "salt": "..."}
]
EOF

# Initialize airdrop
lez-cli private-airdrop initialize \
  --token-definition-id <TOKEN_ID> \
  --allocations allocations.json \
  --network testnet
```

### Step 4: Distribute Claim Packages

Send each recipient their allocation and Merkle proof via secure channel.

### Step 5: Recipients Claim

```bash
lez-cli private-airdrop claim \
  --airdrop-definition-id <AIRDROP_ID> \
  --claim-package claim_package.json \
  --network testnet
```

## Compute Unit Benchmarks

To measure CU costs on LEZ devnet/testnet:

```bash
# Enable CU metering
RISC0_DEV_MODE=0 lez-cli program benchmark \
  --program private_airdrop \
  --operation initialize \
  --network devnet

RISC0_DEV_MODE=0 lez-cli program benchmark \
  --program private_airdrop \
  --operation claim \
  --network devnet
```

Expected costs (approximate, depends on tree depth):
- `initialize`: ~5,000 CU
- `claim`: ~15,000 CU (includes proof verification)

## Security Considerations

1. **Salt Generation**: Use cryptographically secure random salts for each allocation
2. **NSK Protection**: Recipients must keep their nullifier secret key private
3. **Secure Distribution**: Claim packages should be sent over encrypted channels
4. **Proof Verification**: Always verify ZK proofs on-chain before processing claims
5. **Nullifier Registry**: Maintain on-chain registry of used nullifiers

## Known Limitations

1. **Static Eligibility**: Cannot add/remove recipients after commitment
2. **Distributor Knowledge**: Distributor knows full eligibility list at setup
3. **No Amount Hiding**: Individual claim amounts could potentially be inferred
4. **Tree Depth**: Fixed tree depth limits maximum recipients (configurable)

## Future Improvements

1. Dynamic eligibility updates via accumulator-based schemes
2. Batch claiming for gas efficiency
3. Integration with shielded token transfers
4. Time-locked claims
5. Multi-token airdrops

## License

MIT or Apache-2.0 (see LICENSE files)

## Contributing

Contributions welcome! Please open issues for bugs or feature requests.
