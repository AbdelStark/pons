use clap::{Parser, Subcommand};
use std::path::Path;
use tracing::{error, info};

mod keygen;
mod proof;
mod signature;
mod types;

use keygen::*;
use proof::*;
use signature::*;

#[derive(Parser)]
#[command(name = "pons-cli")]
#[command(about = "CLI tool for generating STARK proofs of Bitcoin signature validity")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Taproot keypair
    Keygen {
        /// Output file for the keypair (JSON format)
        #[arg(short, long, default_value = "keypair.json")]
        output: String,
    },
    /// Sign a message with a private key
    Sign {
        /// Path to keypair JSON file
        #[arg(short, long)]
        keypair: String,
        /// Message to sign
        #[arg(short, long)]
        message: String,
        /// Output file for signature (JSON format)
        #[arg(short, long, default_value = "signature.json")]
        output: String,
    },
    /// Generate a STARK proof for signature verification
    Prove {
        /// Path to keypair JSON file
        #[arg(short, long)]
        keypair: String,
        /// Path to signature JSON file
        #[arg(short, long)]
        signature: String,
        /// Output file for the STARK proof
        #[arg(short, long, default_value = "proof.json")]
        output: String,
    },
    /// Verify a STARK proof
    Verify {
        /// Path to proof JSON file
        #[arg(short, long)]
        proof: String,
    },
    /// All-in-one: generate keypair, sign message, and create proof
    Demo {
        /// Message to sign and prove
        #[arg(short, long, default_value = "Hello, Bitcoin!")]
        message: String,
        /// Output directory for generated files
        #[arg(short, long, default_value = ".")]
        output_dir: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen { output } => {
            info!("Generating new Taproot keypair...");

            let keypair = generate_taproot_keypair()?;
            save_keypair_to_file(&keypair, &output)?;

            println!();
            println!("🎉 Keypair generation complete!");
            println!("📁 Saved to: {output}");
            println!("🔑 Public key: {}", keypair.public_key);
            println!();
            println!("⚠️  Keep your private key secure! Anyone with access to the keypair file can spend your Bitcoin.");
        }

        Commands::Sign {
            keypair,
            message,
            output,
        } => {
            info!("Signing message with keypair from: {}", keypair);

            // Load keypair
            let keypair_data = load_keypair_from_file(&keypair)?;

            // Sign the message
            let signature_data = sign_message(&keypair_data, &message)?;

            // Verify the signature (sanity check)
            let is_valid = verify_signature(&keypair_data, &signature_data)?;
            if !is_valid {
                error!("Signature verification failed - this should not happen!");
                anyhow::bail!("Internal error: signature verification failed");
            }

            // Save signature
            save_signature_to_file(&signature_data, &output)?;

            println!();
            println!("✅ Message signed successfully!");
            println!("📝 Message: '{message}'");
            println!("📁 Signature saved to: {output}");
            println!("✍️  Signature: {}", signature_data.signature);
        }

        Commands::Prove {
            keypair,
            signature,
            output,
        } => {
            info!("Generating STARK proof...");

            // Load keypair and signature
            let keypair_data = load_keypair_from_file(&keypair)?;
            let signature_data = load_signature_from_file(&signature)?;

            // Verify the signature matches the keypair
            let is_valid = verify_signature(&keypair_data, &signature_data)?;
            if !is_valid {
                error!("Signature verification failed - signature does not match keypair");
                anyhow::bail!("Invalid signature: does not match the provided keypair");
            }

            println!("✅ Signature verified locally");
            println!("🔧 Generating STARK proof...");

            // Generate STARK proof
            let proof_response = generate_proof(&keypair_data, &signature_data, &output)?;

            if proof_response.success {
                println!("🎉 STARK proof generated successfully!");
                println!("📁 Proof saved to: {output}");

                // Optionally verify the proof immediately
                if verify_proof(&output)? {
                    println!("✅ Proof verification successful!");
                } else {
                    println!("⚠️  Proof verification failed");
                }
            } else {
                error!(
                    "Proof generation failed: {}",
                    proof_response.error.unwrap_or_default()
                );
                anyhow::bail!("Failed to generate STARK proof");
            }
        }

        Commands::Verify { proof } => {
            info!("Verifying STARK proof from: {}", proof);

            if !Path::new(&proof).exists() {
                anyhow::bail!("Proof file not found: {}", proof);
            }

            let is_valid = verify_proof(&proof)?;

            if is_valid {
                println!("✅ Proof verification successful!");
                println!("🎉 The STARK proof is valid!");
            } else {
                println!("❌ Proof verification failed!");
                println!("🚨 The STARK proof is invalid!");
            }
        }

        Commands::Demo {
            message,
            output_dir,
        } => {
            info!("Running complete demonstration...");

            // Create output directory if it doesn't exist
            if !Path::new(&output_dir).exists() {
                std::fs::create_dir_all(&output_dir)?;
            }

            let keypair_path = format!("{output_dir}/demo_keypair.json");
            let signature_path = format!("{output_dir}/demo_signature.json");
            let proof_path = format!("{output_dir}/demo_proof.json");

            println!("🚀 Starting PONS demonstration...");
            println!("📝 Message: '{message}'");
            println!("📁 Output directory: {output_dir}");
            println!();

            // Step 1: Generate keypair
            println!("Step 1: Generating Taproot keypair...");
            let keypair = generate_taproot_keypair()?;
            save_keypair_to_file(&keypair, &keypair_path)?;
            println!();

            // Step 2: Sign message
            println!("Step 2: Signing message...");
            let signature_data = sign_message(&keypair, &message)?;
            save_signature_to_file(&signature_data, &signature_path)?;
            println!();

            // Step 3: Generate STARK proof
            println!("Step 3: Generating STARK proof...");
            let proof_response = generate_proof(&keypair, &signature_data, &proof_path)?;

            if proof_response.success {
                println!("✅ STARK proof generated!");
                println!();

                // Step 4: Verify proof
                println!("Step 4: Verifying STARK proof...");
                if verify_proof(&proof_path)? {
                    println!("✅ Proof verification successful!");
                } else {
                    println!("⚠️  Proof verification failed");
                }

                println!();
                println!("🎉 Demonstration complete!");
                println!("📁 Generated files:");
                println!("   - Keypair: {keypair_path}");
                println!("   - Signature: {signature_path}");
                println!("   - Proof: {proof_path}");
            } else {
                error!(
                    "Failed to generate STARK proof: {}",
                    proof_response.error.unwrap_or_default()
                );
                println!();
                println!("ℹ️  This is expected if cairo-prove is not installed.");
                println!("📖 See SETUP.md for installation instructions.");
            }
        }
    }

    Ok(())
}
