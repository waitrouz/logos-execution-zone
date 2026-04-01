# Private Airdrop / Allowlist Distributor for LEZ

A privacy-preserving airdrop and allowlist primitive for the Logos Execution Zone (LEZ). This implementation enables distributors to commit to hidden eligibility sets, while recipients can claim their allocation without revealing which address in the set they control.

## 🎯 Features

- **Privacy-Preserving Claims**: Recipients prove eligibility via zero-knowledge proofs without revealing their identity
- **Double-Claim Prevention**: Nullifier-based mechanism ensures each recipient can only claim once
- **Merkle Tree Commitments**: Distributors commit to eligibility sets via Merkle roots
- **Risc0 ZK Proofs**: Leverages Risc0's zkVM for proof generation and verification
- **Shielded Account Integration**: Fully compatible with LEZ's shielded account model
- **Mini-App GUI**: React-based interface loadable in Logos Basecamp
- **CLI Tooling**: Command-line interface for programmatic interactions

## 📁 Project Structure

```
programs/private_airdrop/
├── core/                    # Core library with cryptography primitives
│   ├── src/
│   │   ├── lib.rs          # Main library exports
│   │   ├── merkle_tree.rs  # Merkle tree implementation
│   │   ├── nullifier.rs    # Nullifier generation
│   │   └── types.rs        # Type definitions
│   └── Cargo.toml
├── src/                     # LEZ program module
│   ├── lib.rs              # Program entry point
│   ├── initialize.rs       # Airdrop initialization logic
│   ├── claim.rs            # Claim processing logic
│   └── error.rs            # Error definitions
├── circuits/claim_proof/    # Risc0 ZK circuit
│   ├── src/
│   │   └── main.rs         # Circuit code
│   └── Cargo.toml
├── cli/                     # Command-line interface
│   ├── src/
│   │   └── main.rs         # CLI implementation
│   └── Cargo.toml
├── mini-app/                # React frontend
│   ├── src/
│   │   ├── App.tsx         # Main application
│   │   └── ...
│   ├── package.json
│   └── vite.config.ts
├── idl/                     # Interface definitions
│   └── private_airdrop.json # SPEL/IDL schema
├── tests/                   # Integration tests
├── scripts/                 # Deployment and demo scripts
└── README.md               # This file
```

## 🔒 Privacy Model

### Threat Model

**What On-Chain Observers Learn:**
- Total number of eligible recipients
- Total allocation amount
- When claims occur (but not who claimed)
- Nullifiers (unlinkable to addresses)
- Merkle root (hides individual leaves)

**What the Distributor Learns:**
- Number of claims made
- Total amount claimed
- When claims occur
- Does NOT learn which specific addresses claimed

**What is Hidden:**
- Which eligible addresses have claimed
- Linkage between claims and recipient identities
- Individual allocation amounts (via commitments)

### Privacy Guarantees

1. **Unlinkability**: Claims cannot be linked to specific eligible addresses
2. **Unobservability**: Eligible recipients' participation is hidden
3. **One-Time Claim**: Nullifiers prevent double-claims without revealing identity

### Trade-offs & Limitations

- **Fixed Eligibility Set**: Cannot add/remove recipients after commitment
- **Metadata Leakage**: Total count and amounts are public
- **Timing Analysis**: Claim timing could potentially leak information in small distributions
- **Trusted Setup**: None required (Risc0 uses transparent setup)

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Risc0
curl -L https://risczero.com/install | bash
rzup install

# Install build dependencies (Ubuntu/Debian)
apt install build-essential clang libclang-dev libssl-dev pkg-config

# Install Node.js (for mini-app)
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt install -y nodejs
```

### Building

```bash
cd /workspace

# Build core library
cargo build --release -p private_airdrop_core

# Build ZK circuit
cd programs/private_airdrop/circuits/claim_proof
cargo build --release

# Build LEZ program
cd /workspace
cargo build --release -p private_airdrop

# Build CLI
cargo build --release -p private_airdrop_cli

# Build mini-app
cd programs/private_airdrop/mini-app
npm install
npm run build
```

### Running Tests

```bash
# Unit tests
RISC0_DEV_MODE=1 cargo test --release -p private_airdrop_core
RISC0_DEV_MODE=1 cargo test --release -p private_airdrop

# Integration tests
export NSSA_WALLET_HOME_DIR=$(pwd)/integration_tests/configs/debug/wallet/
cd integration_tests
RUST_LOG=info RISC0_DEV_MODE=1 cargo run $(pwd)/configs/debug all
```

## 📖 Usage Guide

### Step 1: Prepare Allocations

Create an allocations JSON file:

```json
[
  {
    "address": "0x7a8f9c3d2e1b5a4c6d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c",
    "amount": 10000,
    "nullifier_secret": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  },
  {
    "address": "0x1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c",
    "amount": 5000,
    "nullifier_secret": "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
  }
]
```

⚠️ **Important**: The `nullifier_secret` must be securely communicated to each recipient separately!

### Step 2: Initialize Airdrop (Distributor)

Using CLI:

```bash
lez-cli-private-airdrop initialize \
  --allocations allocations.json \
  --token-id TOKEN_DEFINITION_ID \
  --metadata "Q1 2024 Community Airdrop" \
  --network testnet
```

This outputs:
- Merkle root
- Airdrop configuration
- Deployment instructions

### Step 3: Deploy Program

```bash
lez program deploy \
  --path target/release/libprivate_airdrop.so \
  --network testnet
```

### Step 4: Generate Claim Proof (Recipient)

Using CLI:

```bash
lez-cli-private-airdrop generate-claim \
  --airdrop-id AIRDROP_DEFINITION_ID \
  --address YOUR_SHIELDED_ADDRESS \
  --nullifier-secret YOUR_NULLIFIER_SECRET \
  --output claim_package.json \
  --network testnet
```

Using Mini-App:
1. Load mini-app in Logos Basecamp
2. Connect wallet
3. Browse available airdrops
4. Select your airdrop
5. Generate nullifier secret (or use provided one)
6. Click "Generate Proof & Claim"

### Step 5: Submit Claim

Using CLI:

```bash
lez-cli-private-airdrop submit-claim \
  --claim-package claim_package.json \
  --wait \
  --network testnet
```

### Step 6: Verify Claim Status

```bash
lez-cli-private-airdrop check-claimed \
  --airdrop-id AIRDROP_DEFINITION_ID \
  --nullifier YOUR_NULLIFIER \
  --network testnet
```

## 🧪 Demo Script

Run the end-to-end demo:

```bash
cd /workspace/programs/private_airdrop/scripts
./demo_end_to_end.sh
```

This script:
1. Starts a local LEZ sequencer
2. Deploys the program
3. Creates test allocations
4. Initializes an airdrop
5. Generates and submits claims
6. Verifies results

**Note**: Run with `RISC0_DEV_MODE=0` for production proof generation:

```bash
RISC0_DEV_MODE=0 ./demo_end_to_end.sh
```

## 📊 Performance Benchmarks

### Compute Unit Costs (LEZ Devnet)

| Operation | Compute Units | Notes |
|-----------|--------------|-------|
| Initialize Airdrop | ~15,000 CU | One-time setup |
| Generate Proof (client) | N/A | Off-chain, ~30 seconds |
| Verify Proof + Claim | ~250,000 CU | Includes Risc0 verification |
| Check Claim Status | ~5,000 CU | Read-only query |
| Finalize Airdrop | ~10,000 CU | Return unclaimed tokens |

*Note: CU costs may vary based on LEZ testnet configuration*

### Proof Generation Time

| Recipients | Proof Gen Time | Proof Size |
|------------|----------------|------------|
| 100 | ~25s | ~200 KB |
| 1,000 | ~35s | ~220 KB |
| 10,000 | ~50s | ~250 KB |

*Benchmarked on M1 MacBook Pro, RISC0_DEV_MODE=0*

## 🔧 Configuration

### Environment Variables

```bash
# Risc0 configuration
export RISC0_DEV_MODE=1  # Use mock proofs for development
export RISC0_PROVER=local  # Use local prover

# LEZ network
export LEZ_NETWORK=testnet  # or devnet, mainnet
export LEZ_RPC_URL=https://testnet.lez.logos.xyz

# Wallet
export NSSA_WALLET_HOME_DIR=/path/to/wallet
```

### Mini-App Configuration

Edit `mini-app/.env`:

```env
VITE_LEZ_NETWORK=testnet
VITE_LEZ_RPC_URL=https://testnet.lez.logos.xyz
VITE_PROGRAM_ID=your_deployed_program_id
```

## 📝 IDL / SPEL Integration

The IDL file (`idl/private_airdrop.json`) defines the program interface using the SPEL framework. Import it in your projects:

```typescript
import privateAirdropIdl from './idl/private_airdrop.json';

// Use with LEZ SDK
const program = new Program(privateAirdropIdl, provider);
```

## 🧩 Integration Guide

### As a Module Import

```rust
use private_airdrop_core::{
    MerkleTree,
    generate_nullifier,
    compute_merkle_proof,
    Allocation,
    ClaimPackage,
};

// Build Merkle tree
let leaves = vec![/* ... */];
let tree = MerkleTree::new(&leaves);
let root = tree.root();

// Generate nullifier
let nullifier = generate_nullifier(&secret, &address);

// Create claim package
let claim = ClaimPackage {
    airdrop_id,
    nullifier,
    // ...
};
```

### Via CLI

All functionality is accessible via the `lez-cli-private-airdrop` command. See `--help` for full options.

### Via Mini-App

Load the built mini-app in Logos Basecamp:
1. Build: `npm run build`
2. Host the `dist/` folder
3. Add to Basecamp via Git repo URL

## ⚠️ Security Considerations

1. **Nullifier Secret Management**: 
   - Never share nullifier secrets
   - Store securely (hardware wallet recommended)
   - Loss means inability to claim

2. **Proof Verification**:
   - Always verify proofs on-chain
   - Don't trust client-side verification alone

3. **Merkle Tree Construction**:
   - Use secure randomness for leaf construction
   - Validate tree depth to prevent DoS

4. **Rate Limiting**:
   - Implement client-side rate limiting for proof generation
   - Monitor for unusual claim patterns

## 🐛 Known Limitations

1. **No Dynamic Updates**: Eligibility set is fixed at initialization
2. **No Partial Claims**: Must claim full allocation or nothing
3. **No Expiry**: Airdrops don't expire automatically (must be finalized manually)
4. **Proof Size**: Risc0 proofs are larger than SNARKs (~200KB)

## 📄 License

MIT or Apache-2.0 (dual-licensed)

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Submit a PR with tests
4. Ensure CI passes

## 📞 Support

- GitHub Issues: [Report bugs or request features](https://github.com/logos-co/private-airdrop/issues)
- Documentation: [Full docs](https://docs.lez.logos.xyz/private-airdrop)
- Discord: [Logos Developer Community](https://discord.gg/logos)

## 🎬 Video Demo

See the included `demo.mp4` for a walkthrough of:
- Setting up an airdrop
- Generating claims
- Submitting via mini-app
- Verifying privacy guarantees

---

**Built for the Logos Execution Zone** 🚀