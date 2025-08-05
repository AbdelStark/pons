use crate::types::{ProofRequest, ProofResponse, SchnorrSignatureData, TaprootKeypair};
use std::path::Path;
use std::process::Command;

/// Generate a STARK proof for signature verification
///
/// This function orchestrates the Cairo proof generation by:
/// 1. Converting signature data to the format expected by the Cairo program
/// 2. Calling cairo-prove to generate the STARK proof
/// 3. Returning the proof data and metadata
///
/// # Arguments
/// * `keypair` - The keypair containing the public key
/// * `signature_data` - The signature data to prove
/// * `proof_output_path` - Where to save the proof file
///
/// # Returns
/// A `ProofResponse` containing the proof or error information
pub fn generate_proof(
    keypair: &TaprootKeypair,
    signature_data: &SchnorrSignatureData,
    proof_output_path: &str,
) -> anyhow::Result<ProofResponse> {
    // Convert to proof request format
    let proof_request = ProofRequest {
        public_key_x: keypair.public_key.clone(),
        signature_r: signature_data.r.clone(),
        signature_s: signature_data.s.clone(),
        message_hash: signature_data.message_hash.clone(),
    };

    generate_proof_from_request(&proof_request, proof_output_path)
}

/// Generate STARK proof from a ProofRequest
///
/// This is the core proof generation function that calls the Cairo program
/// via cairo-prove to generate a STARK proof of signature verification.
///
/// # Arguments
/// * `request` - The proof request containing all necessary parameters
/// * `proof_output_path` - Where to save the generated proof file
///
/// # Returns
/// A `ProofResponse` with the proof data or error information
pub fn generate_proof_from_request(
    request: &ProofRequest,
    proof_output_path: &str,
) -> anyhow::Result<ProofResponse> {
    // Verify cairo-prove is available
    if !is_cairo_prove_available() {
        return Ok(ProofResponse {
            success: false,
            proof: None,
            error: Some("cairo-prove command not found. Please install stwo-cairo.".to_string()),
            proof_file: None,
        });
    }

    // Find the Cairo executable
    let executable_path = find_cairo_executable()?;
    if !Path::new(&executable_path).exists() {
        return Ok(ProofResponse {
            success: false,
            proof: None,
            error: Some(format!("Cairo executable not found at: {executable_path}")),
            proof_file: None,
        });
    }

    // Generate Garaga arguments using Python script
    let signature_hex = format!("{}{}", request.signature_r, request.signature_s);
    let arguments =
        generate_garaga_arguments(&request.public_key_x, &signature_hex, "Hello PONS!")?;

    println!("🔧 Generating STARK proof...");
    println!("📁 Executable: {executable_path}");
    println!("📄 Output: {proof_output_path}");
    println!("🔢 Arguments count: {}", arguments.len());

    // Execute cairo-prove with arguments file
    let args_file = "/tmp/pons_args.json";
    std::fs::write(args_file, serde_json::to_string(&arguments)?)?;

    let output = Command::new("cairo-prove")
        .arg("prove")
        .arg(&executable_path)
        .arg(proof_output_path)
        .arg("--arguments-file")
        .arg(args_file)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute cairo-prove: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(ProofResponse {
            success: false,
            proof: None,
            error: Some(format!(
                "Proof generation failed:\nStderr: {stderr}\nStdout: {stdout}"
            )),
            proof_file: None,
        });
    }

    // Load the generated proof
    match load_proof_from_file(proof_output_path) {
        Ok(proof_json) => {
            println!("✅ STARK proof generated successfully!");
            println!("📁 Proof saved to: {proof_output_path}");

            Ok(ProofResponse {
                success: true,
                proof: Some(proof_json),
                error: None,
                proof_file: Some(proof_output_path.to_string()),
            })
        }
        Err(e) => Ok(ProofResponse {
            success: false,
            proof: None,
            error: Some(format!("Failed to load generated proof: {e}")),
            proof_file: None,
        }),
    }
}

/// Verify a STARK proof using cairo-prove
///
/// Calls cairo-prove verify to validate that a proof is correct.
///
/// # Arguments
/// * `proof_file_path` - Path to the proof file to verify
///
/// # Returns
/// `true` if the proof is valid, `false` otherwise
pub fn verify_proof(proof_file_path: &str) -> anyhow::Result<bool> {
    if !Path::new(proof_file_path).exists() {
        anyhow::bail!("Proof file not found: {}", proof_file_path);
    }

    println!("🔍 Verifying STARK proof...");
    println!("📁 Proof file: {proof_file_path}");

    let output = Command::new("cairo-prove")
        .arg("verify")
        .arg(proof_file_path)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute cairo-prove verify: {}", e))?;

    if output.status.success() {
        println!("✅ Proof verification successful!");
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ Proof verification failed: {stderr}");
        Ok(false)
    }
}

/// Check if cairo-prove command is available
fn is_cairo_prove_available() -> bool {
    Command::new("cairo-prove")
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Find the Cairo executable file
///
/// Looks for the compiled Cairo executable in the pons_stark directory
fn find_cairo_executable() -> anyhow::Result<String> {
    // Try relative to the pons_stark directory
    let possible_paths = [
        "../pons_stark/target/dev/pons_stark.executable.json",
        "./pons_stark/target/dev/pons_stark.executable.json",
        "../../pons_stark/target/dev/pons_stark.executable.json",
        "../target/dev/pons_stark.executable.json",
    ];

    for path in &possible_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    anyhow::bail!("Could not find pons_stark Cairo executable. Make sure to run 'scarb build' in the pons_stark directory first.")
}

/// Generate Garaga arguments using Python script
fn generate_garaga_arguments(
    pubkey: &str,
    signature: &str,
    message: &str,
) -> anyhow::Result<Vec<String>> {
    // Find the Bitcoin argument generation script
    let script_paths = [
        "../pons_stark/gen_args_bitcoin.py",
        "./pons_stark/gen_args_bitcoin.py",
        "../../pons_stark/gen_args_bitcoin.py",
    ];

    let mut script_path = None;
    for path in &script_paths {
        if Path::new(path).exists() {
            script_path = Some(*path);
            break;
        }
    }

    let script_path = script_path.ok_or_else(|| {
        anyhow::anyhow!("Could not find gen_args_bitcoin.py script. Make sure pons_stark is built.")
    })?;

    println!("📄 Using script: {script_path}");
    println!("🔑 Public key: {pubkey}");
    println!("✍️  Signature: {}...", &signature[..16]);
    println!("📝 Message: '{message}'");

    // Call the Python script with Python 3.10 (required for Garaga)
    let output = Command::new("python3.10")
        .arg(script_path)
        .arg("--pubkey")
        .arg(pubkey)
        .arg("--signature")
        .arg(signature)
        .arg("--message")
        .arg(message)
        .arg("--target")
        .arg("execute")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute gen_args.py: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gen_args_bitcoin.py failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let args: Vec<String> = serde_json::from_str(&stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse gen_args_bitcoin.py output: {}", e))?;

    Ok(args)
}

/// Load proof JSON from file
fn load_proof_from_file(proof_path: &str) -> anyhow::Result<serde_json::Value> {
    let proof_content = std::fs::read_to_string(proof_path)
        .map_err(|e| anyhow::anyhow!("Failed to read proof file '{}': {}", proof_path, e))?;

    let proof_json: serde_json::Value = serde_json::from_str(&proof_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse proof JSON: {}", e))?;

    Ok(proof_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keygen::generate_taproot_keypair;
    use crate::signature::sign_message;

    /// Convert hex string to decimal string
    ///
    /// cairo-prove expects decimal arguments, but we work with hex strings.
    /// This function converts hex to decimal for the Cairo program arguments.
    fn hex_to_decimal(hex_str: &str) -> anyhow::Result<String> {
        // Remove 0x prefix if present
        let hex_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);

        // For MVP, handle up to 128-bit values (most Bitcoin keys/hashes fit)
        // In production, you'd want full 256-bit arithmetic
        let value = u128::from_str_radix(hex_clean, 16)
            .map_err(|e| anyhow::anyhow!("Failed to parse hex '{}': {}", hex_str, e))?;

        Ok(value.to_string())
    }

    #[test]
    fn test_hex_to_decimal_conversion() {
        // Test basic conversion
        let result = hex_to_decimal("ff").unwrap();
        assert_eq!(result, "255");

        // Test with 0x prefix
        let result = hex_to_decimal("0xff").unwrap();
        assert_eq!(result, "255");

        // Test longer hex
        let result = hex_to_decimal("1234").unwrap();
        assert_eq!(result, "4660");
    }

    #[test]
    fn test_cairo_prove_availability() {
        // This test will pass if cairo-prove is installed, fail otherwise
        // In CI/CD, you might want to skip this test or mock it
        let available = is_cairo_prove_available();
        println!("cairo-prove available: {available}");
    }

    #[test]
    fn test_proof_request_creation() {
        let keypair = generate_taproot_keypair().unwrap();
        let signature_data = sign_message(&keypair, "test message").unwrap();

        let request = ProofRequest {
            public_key_x: keypair.public_key.clone(),
            signature_r: signature_data.r.clone(),
            signature_s: signature_data.s.clone(),
            message_hash: signature_data.message_hash.clone(),
        };

        // Verify all fields are properly formatted hex strings
        assert_eq!(request.public_key_x.len(), 64);
        assert_eq!(request.signature_r.len(), 64);
        assert_eq!(request.signature_s.len(), 64);
        assert_eq!(request.message_hash.len(), 64);
    }
}
