use crate::types::{SchnorrSignatureData, TaprootKeypair};
use bitcoin::secp256k1::{Keypair, Message, Secp256k1};
use sha2::{Digest, Sha256};

/// Sign a message using BIP-340 Schnorr signatures
///
/// Creates a Schnorr signature over the SHA256 hash of the provided message
/// using the private key from the keypair. This follows the BIP-340 standard
/// used in Bitcoin Taproot.
///
/// # Arguments
/// * `keypair` - The keypair containing the private key for signing
/// * `message` - The message string to sign
///
/// # Returns
/// A `SchnorrSignatureData` containing the signature components and metadata
///
/// # Example
/// ```
/// let keypair = generate_taproot_keypair()?;
/// let signature_data = sign_message(&keypair, "Hello, Bitcoin!")?;
/// println!("Signature: {}", signature_data.signature);
/// ```
pub fn sign_message(
    keypair: &TaprootKeypair,
    message: &str,
) -> anyhow::Result<SchnorrSignatureData> {
    let secp = Secp256k1::new();

    // Get the private key from the keypair
    let secret_key = keypair.get_secret_key()?;
    let keypair_secp = Keypair::from_secret_key(&secp, &secret_key);

    // Hash the message with SHA256
    let message_hash = Sha256::digest(message.as_bytes());
    let message_hash_bytes = message_hash.as_slice();

    // Create a Message object for signing
    let msg = Message::from_digest_slice(message_hash_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create message from hash: {}", e))?;

    // Create BIP-340 Schnorr signature
    let signature = secp.sign_schnorr(&msg, &keypair_secp);
    let signature_bytes = signature.as_ref();

    // Extract R and S components (each 32 bytes)
    let r_bytes = &signature_bytes[0..32];
    let s_bytes = &signature_bytes[32..64];

    Ok(SchnorrSignatureData {
        signature: hex::encode(signature_bytes),
        r: hex::encode(r_bytes),
        s: hex::encode(s_bytes),
        message: message.to_string(),
        message_hash: hex::encode(message_hash_bytes),
    })
}

/// Verify a Schnorr signature (for testing)
///
/// Verifies that a signature is valid for the given message and public key.
/// This is mainly used for testing and validation purposes.
///
/// # Arguments
/// * `keypair` - The keypair containing the public key for verification
/// * `signature_data` - The signature data to verify
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise
pub fn verify_signature(
    keypair: &TaprootKeypair,
    signature_data: &SchnorrSignatureData,
) -> anyhow::Result<bool> {
    let secp = Secp256k1::new();

    // Get the public key
    let public_key = keypair.get_public_key()?;

    // Parse the signature
    let signature_bytes = hex::decode(&signature_data.signature)
        .map_err(|e| anyhow::anyhow!("Failed to decode signature hex: {}", e))?;

    if signature_bytes.len() != 64 {
        anyhow::bail!(
            "Invalid signature length: expected 64 bytes, got {}",
            signature_bytes.len()
        );
    }

    let signature = bitcoin::secp256k1::schnorr::Signature::from_slice(&signature_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to parse signature: {}", e))?;

    // Recreate the message hash
    let expected_hash = Sha256::digest(signature_data.message.as_bytes());
    let expected_hash_hex = hex::encode(expected_hash.as_slice());

    // Verify the message hash matches
    if expected_hash_hex != signature_data.message_hash {
        anyhow::bail!(
            "Message hash mismatch: expected {}, got {}",
            expected_hash_hex,
            signature_data.message_hash
        );
    }

    // Create message for verification
    let msg = Message::from_digest_slice(&expected_hash)
        .map_err(|e| anyhow::anyhow!("Failed to create message from hash: {}", e))?;

    // Verify the signature
    match secp.verify_schnorr(&signature, &msg, &public_key) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Save signature data to JSON file
///
/// Serializes the signature data to JSON format and saves to the specified file.
///
/// # Arguments
/// * `signature_data` - The signature data to save
/// * `output_path` - Path where to save the JSON file
pub fn save_signature_to_file(
    signature_data: &SchnorrSignatureData,
    output_path: &str,
) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(signature_data)?;
    std::fs::write(output_path, json)?;

    println!("✅ Signature saved to: {output_path}");
    println!("📝 Message: '{}'", signature_data.message);
    println!("✍️  Signature: {}", signature_data.signature);

    Ok(())
}

/// Load signature data from JSON file
///
/// Reads and deserializes signature data from a JSON file.
///
/// # Arguments
/// * `input_path` - Path to the JSON file containing the signature data
///
/// # Returns
/// The deserialized `SchnorrSignatureData`
pub fn load_signature_from_file(input_path: &str) -> anyhow::Result<SchnorrSignatureData> {
    let json = std::fs::read_to_string(input_path)
        .map_err(|e| anyhow::anyhow!("Failed to read signature file '{}': {}", input_path, e))?;

    let signature_data: SchnorrSignatureData = serde_json::from_str(&json)
        .map_err(|e| anyhow::anyhow!("Failed to parse signature JSON: {}", e))?;

    // Basic validation
    if signature_data.signature.len() != 128 {
        // 64 bytes * 2 hex chars
        anyhow::bail!(
            "Invalid signature length: expected 128 hex chars, got {}",
            signature_data.signature.len()
        );
    }

    if signature_data.r.len() != 64 {
        // 32 bytes * 2 hex chars
        anyhow::bail!(
            "Invalid R component length: expected 64 hex chars, got {}",
            signature_data.r.len()
        );
    }

    if signature_data.s.len() != 64 {
        // 32 bytes * 2 hex chars
        anyhow::bail!(
            "Invalid S component length: expected 64 hex chars, got {}",
            signature_data.s.len()
        );
    }

    Ok(signature_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keygen::generate_taproot_keypair;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sign_and_verify_message() {
        let keypair = generate_taproot_keypair().unwrap();
        let message = "Hello, Bitcoin!";

        // Sign the message
        let signature_data = sign_message(&keypair, message).unwrap();

        // Verify signature components
        assert_eq!(signature_data.signature.len(), 128); // 64 bytes in hex
        assert_eq!(signature_data.r.len(), 64); // 32 bytes in hex
        assert_eq!(signature_data.s.len(), 64); // 32 bytes in hex
        assert_eq!(signature_data.message, message);
        assert_eq!(signature_data.message_hash.len(), 64); // 32 bytes in hex

        // Verify the signature
        let is_valid = verify_signature(&keypair, &signature_data).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_save_and_load_signature() {
        let keypair = generate_taproot_keypair().unwrap();
        let signature_data = sign_message(&keypair, "Test message").unwrap();

        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        save_signature_to_file(&signature_data, temp_path).unwrap();

        // Load back from file
        let loaded_signature = load_signature_from_file(temp_path).unwrap();

        // Verify they match
        assert_eq!(signature_data.signature, loaded_signature.signature);
        assert_eq!(signature_data.r, loaded_signature.r);
        assert_eq!(signature_data.s, loaded_signature.s);
        assert_eq!(signature_data.message, loaded_signature.message);
        assert_eq!(signature_data.message_hash, loaded_signature.message_hash);
    }

    #[test]
    fn test_signature_verification_with_wrong_key() {
        let keypair1 = generate_taproot_keypair().unwrap();
        let keypair2 = generate_taproot_keypair().unwrap();

        let signature_data = sign_message(&keypair1, "Test message").unwrap();

        // Verification with wrong key should fail
        let is_valid = verify_signature(&keypair2, &signature_data).unwrap();
        assert!(!is_valid);
    }
}
