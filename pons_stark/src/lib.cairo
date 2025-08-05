mod bip340;
mod sha256;
use bip340::hash_challenge;
use garaga::definitions::SECP256K1;
use garaga::signatures::schnorr::{SchnorrSignatureWithHint, is_valid_schnorr_signature};

/// PONS STARK Program - Real BIP-340 Schnorr Signature Verification
///
/// This program verifies BIP-340 Schnorr signatures using the Garaga library.
/// It uses pure Cairo elliptic curve operations without syscalls, making it
/// compatible with STARK proving systems.
///
/// The program takes a message hash and a Schnorr signature with MSM hints,
/// then generates a STARK proof that the signature is cryptographically valid.
///
/// Arguments (serialized from SchnorrSignatureWithHint):
/// - message_hash: The SHA256 hash of the message (u256)
/// - Followed by the serialized Garaga SchnorrSignatureWithHint structure
///
/// Returns:
/// - 1 if signature is valid
/// - 0 if signature is invalid

/// Struct representing a Bitcoin signature verification event with hints
#[derive(Drop, Serde)]
pub struct BitcoinSignedMessage {
    /// Message hash (the signed message hash)
    pub message_hash: u256,
    /// Garaga Schnorr signature with MSM hints, public key, and hash challenge
    pub sig_with_hint: SchnorrSignatureWithHint,
}

// ***************************************
// *********** MAIN ENTRYPOINT ***********
// ***************************************
#[executable]
pub fn main(arguments: Array<felt252>) {
    let mut args = arguments.span();
    let events: Array<BitcoinSignedMessage> = Serde::deserialize(ref args)
        .expect('Deserialization failed');

    println!("Verifying {} Bitcoin signature(s)...", events.len());

    verify_signature_batch(events);

    println!("Bitcoin signature verification successful");
}

/// Verifies a batch of Bitcoin message signatures
/// Fails if any of the signatures are invalid
pub fn verify_signature_batch(events: Array<BitcoinSignedMessage>) {
    for event in events {
        // Compute the BIP-340 challenge hash
        let e = hash_challenge(
            event.sig_with_hint.signature.rx, event.sig_with_hint.signature.px, event.message_hash,
        ) % SECP256K1
            .n;

        // Verify the challenge matches
        assert(e == event.sig_with_hint.signature.e, 'Invalid challenge');

        // Verify the Schnorr signature using Garaga
        assert(is_valid_schnorr_signature(event.sig_with_hint, 2), 'Invalid signature');
    }
}
