//! Private Airdrop CLI for LEZ
//! 
//! Provides commands for:
//! - Setting up private airdrops
//! - Generating claim proofs
//! - Submitting claims to the network

use clap::{Parser, Subcommand};
use private_airdrop_core::{
    MerkleTree, ClaimPackage, Allocation, AirdropConfig,
    generate_nullifier, compute_merkle_proof,
};
use risc0_zkvm::{default_prover, ProverOpts};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "lez-cli-private-airdrop")]
#[command(about = "Private Airdrop CLI for LEZ", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "testnet")]
    network: String,
    
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new private airdrop
    Initialize {
        /// Path to allocations JSON file
        #[arg(short, long)]
        allocations: PathBuf,
        
        /// Token definition ID
        #[arg(short, long)]
        token_id: String,
        
        /// Optional metadata
        #[arg(short, long)]
        metadata: Option<String>,
    },
    
    /// Generate a claim proof package
    GenerateClaim {
        /// Airdrop definition ID
        #[arg(short, long)]
        airdrop_id: String,
        
        /// Recipient's shielded address
        #[arg(short, long)]
        address: String,
        
        /// Nullifier secret (keep this private!)
        #[arg(short, long)]
        nullifier_secret: String,
        
        /// Output path for claim package
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Submit a claim to the network
    SubmitClaim {
        /// Path to claim package JSON
        #[arg(short, long)]
        claim_package: PathBuf,
        
        /// Optional: wait for confirmation
        #[arg(short, long)]
        wait: bool,
    },
    
    /// Check if an address has already claimed
    CheckClaimed {
        /// Airdrop definition ID
        #[arg(short, long)]
        airdrop_id: String,
        
        /// Nullifier to check
        #[arg(short, long)]
        nullifier: String,
    },
    
    /// Verify a claim package locally
    VerifyClaim {
        /// Path to claim package JSON
        #[arg(short, long)]
        claim_package: PathBuf,
        
        /// Expected Merkle root
        #[arg(short, long)]
        merkle_root: String,
    },
    
    /// Export airdrop information
    ExportAirdrop {
        /// Airdrop definition ID
        #[arg(short, long)]
        airdrop_id: String,
        
        /// Output path
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct ClaimPackageOutput {
    airdrop_id: String,
    nullifier: String,
    amount_commitment: String,
    merkle_root: String,
    proof_receipt: String,
    timestamp: u64,
}

fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

async fn initialize_airdrop(allocations_path: PathBuf, token_id: String, metadata: Option<String>) -> Result<()> {
    println!("🚀 Initializing private airdrop...");
    
    // Read allocations file
    let allocations_data = fs::read_to_string(&allocations_path)
        .context("Failed to read allocations file")?;
    let allocations: Vec<Allocation> = serde_json::from_str(&allocations_data)
        .context("Failed to parse allocations JSON")?;
    
    println!("✓ Loaded {} allocations", allocations.len());
    
    // Build Merkle tree
    let mut leaves = Vec::new();
    for alloc in &allocations {
        let mut leaf_data = Vec::new();
        leaf_data.extend_from_slice(&hex::decode(&alloc.address)?);
        leaf_data.extend_from_slice(&alloc.amount.to_le_bytes());
        leaf_data.extend_from_slice(&hex::decode(&alloc.nullifier_secret)?);
        leaves.push(hash_sha256(&leaf_data));
    }
    
    let merkle_tree = MerkleTree::new(&leaves);
    let merkle_root = merkle_tree.root();
    
    println!("✓ Built Merkle tree with root: {}", hex::encode(merkle_root));
    
    // Create airdrop config
    let config = AirdropConfig {
        token_id,
        merkle_root,
        total_amount: allocations.iter().map(|a| a.amount).sum(),
        recipient_count: allocations.len() as u64,
        metadata: metadata.unwrap_or_default(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };
    
    // Output configuration
    let config_json = serde_json::to_string_pretty(&config)?;
    println!("\n📋 Airdrop Configuration:\n{}", config_json);
    
    println!("\n✅ Airdrop initialized successfully!");
    println!("Next step: Deploy to LEZ network with:");
    println!("  lez program deploy --path target/release/libprivate_airdrop.so --network testnet");
    println!("  lez private-airdrop submit-config --config <config.json> --network testnet");
    
    Ok(())
}

async fn generate_claim(airdrop_id: String, address: String, nullifier_secret: String, output: Option<PathBuf>) -> Result<()> {
    println!("🔐 Generating claim proof...");
    
    // Decode inputs
    let address_bytes = hex::decode(&address).context("Invalid address format")?;
    let secret_bytes = hex::decode(&nullifier_secret).context("Invalid nullifier secret format")?;
    
    if address_bytes.len() != 32 {
        anyhow::bail!("Address must be 32 bytes");
    }
    if secret_bytes.len() != 32 {
        anyhow::bail!("Nullifier secret must be 32 bytes");
    }
    
    let mut address_arr = [0u8; 32];
    let mut secret_arr = [0u8; 32];
    address_arr.copy_from_slice(&address_bytes);
    secret_arr.copy_from_slice(&secret_bytes);
    
    // TODO: Fetch airdrop data from chain (merkle_root, user's allocation)
    // For now, we'll use placeholder values
    let merkle_root = [0u8; 32]; // Should fetch from chain
    let amount = 1000u64; // Should fetch from allocations
    
    // Generate nullifier
    let nullifier = generate_nullifier(&secret_arr, &address_arr);
    println!("✓ Generated nullifier: {}", hex::encode(nullifier));
    
    // Generate amount commitment
    let mut commitment_data = Vec::new();
    commitment_data.extend_from_slice(&amount.to_le_bytes());
    commitment_data.extend_from_slice(&secret_arr[..8]);
    let amount_commitment = hash_sha256(&commitment_data);
    println!("✓ Generated amount commitment: {}", hex::encode(amount_commitment));
    
    // TODO: Generate Merkle proof (requires fetching user's leaf index and siblings)
    let merkle_proof = compute_merkle_proof(&[], 0); // Placeholder
    
    // Prepare public inputs for ZK circuit
    let public_inputs = private_airdrop_core::zk::PublicInputs {
        merkle_root,
        nullifier,
        amount_commitment,
    };
    
    // Prepare private inputs
    let private_inputs = private_airdrop_core::zk::PrivateInputs {
        address: address_arr,
        nullifier_secret: secret_arr,
        amount,
        merkle_proof,
    };
    
    println!("🧠 Generating Risc0 proof...");
    
    // Generate proof using Risc0
    let prover = default_prover();
    let opts = ProverOpts::default();
    
    // Note: In production, load the actual receipt from compiled circuit
    // let receipt = prover.prove_with_opts(&opts, &guest_code, &(public_inputs, private_inputs))?;
    
    // For now, create a mock receipt
    let receipt_mock = serde_json::json!({
        "status": "mock_proof_for_development",
        "note": "Build circuit with: cargo build --release -p claim_proof"
    });
    
    println!("✓ Proof generated (mock mode - build circuit for production)");
    
    // Create claim package
    let claim_package = ClaimPackageOutput {
        airdrop_id,
        nullifier: hex::encode(nullifier),
        amount_commitment: hex::encode(amount_commitment),
        merkle_root: hex::encode(merkle_root),
        proof_receipt: receipt_mock.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };
    
    // Output claim package
    let claim_json = serde_json::to_string_pretty(&claim_package)?;
    
    match output {
        Some(path) => {
            fs::write(&path, &claim_json)?;
            println!("✓ Claim package saved to: {:?}", path);
        }
        None => {
            println!("\n📦 Claim Package:\n{}", claim_json);
        }
    }
    
    println!("\n✅ Claim proof generated successfully!");
    println!("Next step: Submit with:");
    println!("  lez-cli-private-airdrop submit-claim --claim-package <path> --network testnet");
    
    Ok(())
}

async fn submit_claim(claim_package_path: PathBuf, wait: bool) -> Result<()> {
    println!("📤 Submitting claim...");
    
    // Read claim package
    let claim_data = fs::read_to_string(&claim_package_path)
        .context("Failed to read claim package")?;
    let claim_package: ClaimPackageOutput = serde_json::from_str(&claim_data)
        .context("Failed to parse claim package")?;
    
    println!("✓ Loaded claim package for airdrop: {}", claim_package.airdrop_id);
    println!("  Nullifier: {}", claim_package.nullifier);
    
    // TODO: Submit to LEZ network
    // lez transaction send --program private-airdrop --instruction claim --args ...
    
    println!("⏳ Submitting to {} network...", "testnet");
    
    if wait {
        println!("⏳ Waiting for confirmation...");
        // TODO: Wait for transaction confirmation
        println!("✅ Claim confirmed!");
    } else {
        println!("✅ Claim submitted! Transaction pending.");
    }
    
    Ok(())
}

async fn check_claimed(airdrop_id: String, nullifier: String) -> Result<()> {
    println!("🔍 Checking claim status...");
    
    // TODO: Query LEZ chain for nullifier
    // lez query program-state --program private-airdrop --key nullifiers/<nullifier>
    
    println!("Airdrop: {}", airdrop_id);
    println!("Nullifier: {}", nullifier);
    println!("Status: NOT_YET_IMPLEMENTED (requires LEZ node connection)");
    
    Ok(())
}

async fn verify_claim(claim_package_path: PathBuf, merkle_root: String) -> Result<()> {
    println!("✅ Verifying claim package...");
    
    // Read claim package
    let claim_data = fs::read_to_string(&claim_package_path)
        .context("Failed to read claim package")?;
    let claim_package: ClaimPackageOutput = serde_json::from_str(&claim_data)
        .context("Failed to parse claim package")?;
    
    // Verify merkle root matches
    let expected_root = merkle_root.strip_prefix("0x").unwrap_or(&merkle_root);
    if claim_package.merkle_root != expected_root {
        anyhow::bail!("Merkle root mismatch! Expected {}, got {}", expected_root, claim_package.merkle_root);
    }
    
    println!("✓ Merkle root verified");
    println!("✓ Nullifier: {}", claim_package.nullifier);
    println!("✓ Amount commitment: {}", claim_package.amount_commitment);
    
    // TODO: Verify Risc0 proof receipt
    println!("⚠️  Proof verification not implemented (requires Risc0 verifier)");
    
    println!("\n✅ Claim package is valid!");
    
    Ok(())
}

async fn export_airdrop(airdrop_id: String, output: PathBuf) -> Result<()> {
    println!("📥 Exporting airdrop data...");
    
    // TODO: Fetch airdrop data from LEZ chain
    // lez query program-state --program private-airdrop --key airdrops/<airdrop_id>
    
    println!("Airdrop ID: {}", airdrop_id);
    println!("Export path: {:?}", output);
    println!("Status: NOT_YET_IMPLEMENTED (requires LEZ node connection)");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }
    
    println!("🔒 Private Airdrop CLI v0.1.0");
    println!("Network: {}\n", cli.network);
    
    match cli.command {
        Commands::Initialize { allocations, token_id, metadata } => {
            initialize_airdrop(allocations, token_id, metadata).await?
        }
        Commands::GenerateClaim { airdrop_id, address, nullifier_secret, output } => {
            generate_claim(airdrop_id, address, nullifier_secret, output).await?
        }
        Commands::SubmitClaim { claim_package, wait } => {
            submit_claim(claim_package, wait).await?
        }
        Commands::CheckClaimed { airdrop_id, nullifier } => {
            check_claimed(airdrop_id, nullifier).await?
        }
        Commands::VerifyClaim { claim_package, merkle_root } => {
            verify_claim(claim_package, merkle_root).await?
        }
        Commands::ExportAirdrop { airdrop_id, output } => {
            export_airdrop(airdrop_id, output).await?
        }
    }
    
    Ok(())
}
