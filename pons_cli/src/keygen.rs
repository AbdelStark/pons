use crate::types::TaprootKeypair;
use bitcoin::secp256k1::rand::thread_rng;
use bitcoin::secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey};

/// Generate a new Taproot keypair
///
/// Creates a cryptographically secure random private key and derives
/// the corresponding X-only public key for use in Taproot transactions.
///
/// # Returns
/// A `TaprootKeypair` containing both the private key and X-only public key
///
/// # Example
/// ```
/// let keypair = generate_taproot_keypair()?;
/// println!("Public key: {}", keypair.public_key);
/// ```
pub fn generate_taproot_keypair() -> anyhow::Result<TaprootKeypair> {
    let secp = Secp256k1::new();

    // Generate cryptographically secure random private key
    let secret_key = SecretKey::new(&mut thread_rng());

    // Derive X-only public key (32 bytes instead of 33)
    // This removes the y-coordinate parity byte for Taproot compatibility
    let public_key = XOnlyPublicKey::from(secret_key.public_key(&secp));

    Ok(TaprootKeypair::new(secret_key, public_key))
}

/// Save keypair to JSON file
///
/// Serializes the keypair to JSON format and saves to the specified file path.
/// The private key is hex-encoded for safe serialization.
///
/// # Arguments
/// * `keypair` - The keypair to save
/// * `output_path` - Path where to save the JSON file
///
/// # Security Note
/// The private key is stored in plain text. In production, consider encryption.
pub fn save_keypair_to_file(keypair: &TaprootKeypair, output_path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(keypair)?;
    std::fs::write(output_path, json)?;

    println!("✅ Keypair saved to: {output_path}");
    println!("🔑 Public key: {}", keypair.public_key);

    Ok(())
}

/// Load keypair from JSON file
///
/// Reads and deserializes a keypair from a JSON file created by `save_keypair_to_file`.
///
/// # Arguments  
/// * `input_path` - Path to the JSON file containing the keypair
///
/// # Returns
/// The deserialized `TaprootKeypair`
pub fn load_keypair_from_file(input_path: &str) -> anyhow::Result<TaprootKeypair> {
    let json = std::fs::read_to_string(input_path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair file '{}': {}", input_path, e))?;

    let keypair: TaprootKeypair = serde_json::from_str(&json)
        .map_err(|e| anyhow::anyhow!("Failed to parse keypair JSON: {}", e))?;

    // Validate that the keys can be parsed
    keypair.get_secret_key()?;
    keypair.get_public_key()?;

    Ok(keypair)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_taproot_keypair() {
        let keypair = generate_taproot_keypair().unwrap();

        // Verify we can parse the keys back
        assert!(keypair.get_secret_key().is_ok());
        assert!(keypair.get_public_key().is_ok());

        // Verify public key is 64 hex characters (32 bytes)
        assert_eq!(keypair.public_key.len(), 64);

        // Verify private key is 64 hex characters (32 bytes)
        assert_eq!(keypair.private_key.len(), 64);
    }

    #[test]
    fn test_save_and_load_keypair() {
        let keypair = generate_taproot_keypair().unwrap();

        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        save_keypair_to_file(&keypair, temp_path).unwrap();

        // Load back from file
        let loaded_keypair = load_keypair_from_file(temp_path).unwrap();

        // Verify they match
        assert_eq!(keypair.private_key, loaded_keypair.private_key);
        assert_eq!(keypair.public_key, loaded_keypair.public_key);
    }
}
