use bitcoin::secp256k1::{SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

/// Represents a Taproot keypair for Bitcoin transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaprootKeypair {
    /// Private key (hex-encoded for serialization)
    pub private_key: String,
    /// X-only public key (hex-encoded)
    pub public_key: String,
}

impl TaprootKeypair {
    /// Create a new TaprootKeypair from secp256k1 keys
    pub fn new(secret_key: SecretKey, public_key: XOnlyPublicKey) -> Self {
        Self {
            private_key: secret_key.display_secret().to_string(),
            public_key: public_key.to_string(),
        }
    }

    /// Get the secret key from hex string
    pub fn get_secret_key(&self) -> anyhow::Result<SecretKey> {
        self.private_key
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))
    }

    /// Get the public key from hex string  
    pub fn get_public_key(&self) -> anyhow::Result<XOnlyPublicKey> {
        self.public_key
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse public key: {}", e))
    }
}

/// Represents a Schnorr signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchnorrSignatureData {
    /// Signature bytes (64 bytes, hex-encoded)
    pub signature: String,
    /// R component (32 bytes, hex-encoded)
    pub r: String,
    /// S component (32 bytes, hex-encoded)
    pub s: String,
    /// Message that was signed
    pub message: String,
    /// Message hash (SHA256 of message)
    pub message_hash: String,
}

/// Request for generating a STARK proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    /// X-only public key (hex-encoded)
    pub public_key_x: String,
    /// Signature R component (hex-encoded)
    pub signature_r: String,
    /// Signature S component (hex-encoded)
    pub signature_s: String,
    /// Message hash (hex-encoded)
    pub message_hash: String,
}

/// Response from proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResponse {
    /// Whether proof generation was successful
    pub success: bool,
    /// STARK proof JSON (if successful)
    pub proof: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Path to proof file
    pub proof_file: Option<String>,
}
