# PONS: Proof of Non-Spam for Bitcoin - Product Requirements Document

## 1. Problem Statement

Bitcoin transactions increasingly contain arbitrary data disguised as legitimate script elements, particularly in Taproot outputs where spammers create fake Taproot trees with branches containing arbitrary data instead of valid script paths. Current filtering mechanisms either:

- Allow spam through by being too permissive
- Break legitimate use cases like BitVM by being too restrictive (e.g., limiting tree size)

## 2. Solution Overview

PONS (Proof of Non-Spam) uses STARK proofs to cryptographically verify that transaction elements contain legitimate data structures (public keys, hashes, script elements) rather than arbitrary spam. Transaction senders generate proofs demonstrating their outputs contain valid cryptographic material, and relays/miners only accept transactions with valid proofs.

## 3. System Architecture

### 3.1 Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   pons_cli      │    │  pons_stark     │    │  pons_relay     │
│   (Rust)        │────│  (Cairo)        │────│  (Rust)         │
│                 │    │                 │    │                 │
│ • Keypair gen   │    │ • Sig verify    │    │ • Proof verify  │
│ • Tx creation   │    │ • STARK proof   │    │ • Tx relay      │
│ • Proof orchestr│    │ • Cairo program │    │ • Network layer │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Bitcoin Network                              │
│  • Taproot transactions with embedded proof commitments        │
│  • Standard P2TR outputs with verified cryptographic material  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Data Flow

1. **Proof Generation**: `pons_cli` → `pons_stark` → STARK proof
2. **Transaction Creation**: `pons_cli` creates Bitcoin transaction with proof commitment
3. **Transaction Relay**: `pons_cli` → `pons_relay` (transaction + proof)
4. **Proof Verification**: `pons_relay` verifies STARK proof
5. **Network Propagation**: `pons_relay` → Bitcoin network (if proof valid)

## 4. Component Specifications

### 4.1 pons_stark (Cairo Program)

**Purpose**: Generate STARK proofs for cryptographic signature verification

**Core Function**:

```cairo
#[executable]
fn main(
    public_key: u256,           // Schnorr public key (32 bytes)
    signature_r: u256,          // Signature R component (32 bytes)
    signature_s: u256,          // Signature S component (32 bytes)
    message_hash: u256          // SHA256 hash of signed message (32 bytes)
) -> felt252 {
    // Verify BIP-340 Schnorr signature
    // Return 1 if valid, 0 if invalid
}
```

**Dependencies**:

- `alexandria_btc` v0.5.1+ for BIP-340 Schnorr signature verification
- Cairo standard library for field arithmetic

**Input Validation**:

- Public key must be valid secp256k1 point
- Signature components must be in valid field range
- Message hash must be 32 bytes

**Output**:

- STARK proof demonstrating valid signature verification
- Proof commitments to public key and message

### 4.2 pons_cli (Rust Binary)

**Purpose**: User-facing tool for generating proofs and creating transactions

**Core Functions**:

1. **Keypair Generation**:

```rust
fn generate_taproot_keypair() -> Result<(SecretKey, XOnlyPublicKey), Error>
```

2. **Message Signing**:

```rust
fn sign_message(
    secret_key: &SecretKey,
    message: &str
) -> Result<Signature, Error>
```

3. **Proof Generation**:

```rust
fn generate_proof(
    public_key: XOnlyPublicKey,
    signature: Signature,
    message: &str
) -> Result<StarkProof, Error>
```

4. **Transaction Creation**:

```rust
fn create_taproot_transaction(
    keypair: Keypair,
    proof: StarkProof,
    output_amount: u64
) -> Result<Transaction, Error>
```

**Dependencies**:

- `bitcoin` v0.32+ for transaction creation and Taproot handling
- `secp256k1` v0.29+ for cryptographic operations
- `tokio` v1.0+ for async networking
- `clap` v4.0+ for CLI interface
- `serde` v1.0+ for proof serialization
- `anyhow` v1.0+ for error handling

**CLI Interface**:

```bash
pons-cli keygen                           # Generate keypair
pons-cli sign --key <key> --msg <msg>     # Sign message
pons-cli prove --pubkey <key> --sig <sig> --msg <msg>  # Generate proof
pons-cli tx --key <key> --proof <proof> --amount <sat> # Create transaction
pons-cli relay --tx <tx> --proof <proof> --relay <url> # Submit to relay
```

### 4.3 pons_relay (Rust Server)

**Purpose**: Network relay that validates proofs before propagating transactions

**Core Functions**:

1. **Proof Verification**:

```rust
async fn verify_stark_proof(proof: &StarkProof) -> Result<bool, Error>
```

2. **Transaction Validation**:

```rust
async fn validate_transaction(
    tx: &Transaction,
    proof: &StarkProof
) -> Result<ValidationResult, Error>
```

3. **Network Relay**:

```rust
async fn relay_transaction(tx: Transaction) -> Result<Txid, Error>
```

**Dependencies**:

- `tokio` v1.0+ for async runtime
- `axum` v0.7+ for HTTP API
- `bitcoin` v0.32+ for transaction parsing
- `bitcoincore-rpc` v0.19+ for Bitcoin Core integration
- `tower` v0.4+ for middleware
- `tracing` v0.1+ for observability

**API Endpoints**:

```
POST /submit
{
  "transaction": "<hex>",
  "proof": "<stark_proof_json>"
}

GET /status/<txid>
Response: {"status": "verified|pending|rejected", "reason": "..."}
```

## 5. Implementation Plan - MVP Iteration 1

### 5.1 Phase 1: Cairo Program Foundation (Week 1)

**Objective**: Implement basic Schnorr signature verification in Cairo

**Tasks**:

1. **Setup Cairo environment**:

   - Extend existing `Scarb.toml` with required dependencies
   - Configure `alexandria_btc` for BIP-340 support

2. **Implement signature verification**:

   ```cairo
   // File: src/lib.cairo
   use alexandria_btc::signature::verify_bip340_signature;
   use alexandria_btc::types::Signature;

   #[executable]
   fn main(
       pubkey: u256,
       sig_r: u256,
       sig_s: u256,
       msg_hash: u256
   ) -> felt252 {
       let signature = Signature { r: sig_r, s: sig_s };
       let is_valid = verify_bip340_signature(msg_hash, signature, pubkey);
       if is_valid { 1 } else { 0 }
   }
   ```

3. **Test proof generation**:
   - Update `prove.sh` to accept signature parameters
   - Verify proof generation with test vectors
   - Validate proof size and generation time

**Success Criteria**:

- Cairo program compiles without errors
- Generates valid STARK proofs for known good signatures
- Rejects proofs for invalid signatures
- Proof generation completes in <30 seconds

### 5.2 Phase 2: Rust CLI Implementation (Week 2)

**Objective**: Build command-line tool for keypair generation and proof orchestration

**Project Structure**:

```
pons_cli/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point
│   ├── keygen.rs        # Taproot keypair generation
│   ├── signature.rs     # Message signing
│   ├── proof.rs         # Cairo proof orchestration
│   ├── transaction.rs   # Bitcoin transaction creation
│   └── types.rs         # Common data structures
```

**Key Files**:

1. **Cargo.toml**:

   ```toml
   [package]
   name = "pons_cli"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   bitcoin = "0.32"
   secp256k1 = { version = "0.29", features = ["rand-std"] }
   clap = { version = "4.0", features = ["derive"] }
   tokio = { version = "1.0", features = ["full"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   anyhow = "1.0"
   hex = "0.4"
   sha2 = "0.10"
   ```

2. **src/keygen.rs**:

   ```rust
   use bitcoin::secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey};
   use bitcoin::secp256k1::rand::thread_rng;

   pub fn generate_keypair() -> anyhow::Result<(SecretKey, XOnlyPublicKey)> {
       let secp = Secp256k1::new();
       let secret_key = SecretKey::new(&mut thread_rng());
       let public_key = XOnlyPublicKey::from(secret_key.public_key(&secp));
       Ok((secret_key, public_key))
   }
   ```

3. **src/signature.rs**:

   ```rust
   use bitcoin::secp256k1::{Message, Secp256k1, SecretKey, Keypair};
   use sha2::{Sha256, Digest};

   pub fn sign_message(
       secret_key: &SecretKey,
       message: &str
   ) -> anyhow::Result<secp256k1::schnorr::Signature> {
       let secp = Secp256k1::new();
       let keypair = Keypair::from_secret_key(&secp, secret_key);
       let msg_hash = Sha256::digest(message.as_bytes());
       let message = Message::from_digest_slice(&msg_hash)?;
       Ok(secp.sign_schnorr(&message, &keypair))
   }
   ```

4. **src/proof.rs**:

   ```rust
   use std::process::Command;
   use serde_json::Value;

   pub struct ProofRequest {
       pub public_key: String,
       pub signature_r: String,
       pub signature_s: String,
       pub message_hash: String,
   }

   pub fn generate_proof(req: ProofRequest) -> anyhow::Result<Value> {
       // Call cairo-prove with signature parameters
       let args = format!("{},{},{},{}",
           req.public_key, req.signature_r, req.signature_s, req.message_hash);

       let output = Command::new("cairo-prove")
           .args(["prove", "../target/dev/pons.executable.json", "./proof.json"])
           .arg("--arguments")
           .arg(&args)
           .output()?;

       if !output.status.success() {
           anyhow::bail!("Proof generation failed: {}",
               String::from_utf8_lossy(&output.stderr));
       }

       let proof: Value = serde_json::from_str(&std::fs::read_to_string("./proof.json")?)?;
       Ok(proof)
   }
   ```

**Success Criteria**:

- CLI generates valid Taproot keypairs
- Creates BIP-340 Schnorr signatures for arbitrary messages
- Successfully orchestrates Cairo proof generation
- Produces JSON-serialized proofs compatible with relay

### 5.3 Phase 3: Rust Relay Implementation (Week 3)

**Objective**: Build HTTP relay that verifies STARK proofs and propagates valid transactions

**Project Structure**:

```
pons_relay/
├── Cargo.toml
├── src/
│   ├── main.rs          # HTTP server entry point
│   ├── api.rs           # REST API handlers
│   ├── verification.rs  # STARK proof verification
│   ├── bitcoin.rs       # Bitcoin Core RPC integration
│   ├── types.rs         # Request/response types
│   └── config.rs        # Configuration management
```

**Key Files**:

1. **Cargo.toml**:

   ```toml
   [package]
   name = "pons_relay"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   tokio = { version = "1.0", features = ["full"] }
   axum = "0.7"
   tower = "0.4"
   tower-http = { version = "0.5", features = ["cors", "trace"] }
   tracing = "0.1"
   tracing-subscriber = "0.3"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   bitcoin = "0.32"
   bitcoincore-rpc = "0.19"
   anyhow = "1.0"
   ```

2. **src/verification.rs**:

   ```rust
   use std::process::Command;
   use serde_json::Value;

   pub async fn verify_stark_proof(proof: &Value) -> anyhow::Result<bool> {
       // Write proof to temporary file
       let proof_file = "/tmp/verify_proof.json";
       std::fs::write(proof_file, serde_json::to_string_pretty(proof)?)?;

       // Call cairo-prove verify
       let output = Command::new("cairo-prove")
           .args(["verify", proof_file])
           .output()?;

       Ok(output.status.success())
   }
   ```

3. **src/api.rs**:

   ```rust
   use axum::{Json, response::Json as ResponseJson};
   use serde::{Deserialize, Serialize};

   #[derive(Deserialize)]
   pub struct SubmitRequest {
       pub transaction: String,    // Hex-encoded transaction
       pub proof: serde_json::Value,  // STARK proof JSON
   }

   #[derive(Serialize)]
   pub struct SubmitResponse {
       pub status: String,
       pub txid: Option<String>,
       pub message: String,
   }

   pub async fn submit_transaction(
       Json(payload): Json<SubmitRequest>
   ) -> ResponseJson<SubmitResponse> {
       // Verify STARK proof
       match crate::verification::verify_stark_proof(&payload.proof).await {
           Ok(true) => {
               // Relay transaction to Bitcoin network
               // Implementation depends on Bitcoin Core RPC setup
               ResponseJson(SubmitResponse {
                   status: "accepted".to_string(),
                   txid: Some("placeholder_txid".to_string()),
                   message: "Transaction accepted and relayed".to_string(),
               })
           }
           Ok(false) => {
               ResponseJson(SubmitResponse {
                   status: "rejected".to_string(),
                   txid: None,
                   message: "Invalid STARK proof".to_string(),
               })
           }
           Err(e) => {
               ResponseJson(SubmitResponse {
                   status: "error".to_string(),
                   txid: None,
                   message: format!("Verification error: {}", e),
               })
           }
       }
   }
   ```

**Success Criteria**:

- HTTP server accepts proof submission requests
- Correctly verifies STARK proofs using cairo-prove
- Rejects transactions with invalid proofs
- Returns appropriate HTTP status codes and error messages

### 5.4 Phase 4: Integration Testing (Week 4)

**Objective**: End-to-end testing of complete proof-of-concept system

**Test Scenarios**:

1. **Happy Path Test**:

   ```bash
   # Generate keypair
   cd pons_cli && cargo run -- keygen --output keypair.json

   # Sign message
   cargo run -- sign --key keypair.json --message "hello world" --output signature.json

   # Generate proof
   cargo run -- prove --pubkey keypair.json --signature signature.json --message "hello world"

   # Start relay
   cd ../pons_relay && cargo run &

   # Submit transaction
   cd ../pons_cli && cargo run -- relay --tx transaction.hex --proof proof.json --relay http://localhost:3000
   ```

2. **Invalid Signature Test**:

   - Generate proof with mismatched signature/message
   - Verify relay rejects transaction
   - Confirm appropriate error messages

3. **Performance Test**:
   - Measure proof generation time
   - Measure proof verification time
   - Measure end-to-end latency

**Success Criteria**:

- Complete end-to-end flow works without manual intervention
- Invalid proofs are correctly rejected
- System handles concurrent requests
- Performance meets acceptable thresholds (<30s proof generation, <5s verification)

## 6. Dependencies and Infrastructure

### 6.1 External Dependencies

**Cairo Prover**:

- `cairo-prove` binary from stwo-cairo project
- Requires Rust toolchain and Scarb 2.10.0+

**Bitcoin Infrastructure**:

- Bitcoin Core node for transaction relay (optional for MVP)
- Testnet/Signet environment for testing

**Development Tools**:

- Rust 1.70+ with Cargo
- Git for version control
- Docker for containerized deployment (future)

### 6.2 Network Requirements

**For Development**:

- Internet access for dependency downloads
- GitHub access for alexandria_btc library

**For Production**:

- Bitcoin network connectivity (P2P or RPC)
- Stable internet for proof verification requests
- Optional: Monitoring and logging infrastructure

## 7. Security Considerations

### 7.1 Cryptographic Security

**STARK Proof Integrity**:

- Proof verification must be deterministic
- No trusted setup required (advantage of STARKs)
- Proof size and verification time must be practical

**Bitcoin Integration**:

- Private key handling follows Bitcoin best practices
- Signature generation uses secure randomness
- Transaction construction prevents malleability

### 7.2 Network Security

**Relay Protection**:

- Rate limiting on proof submission endpoints
- Input validation on all API requests
- DoS protection against computational attacks

**Proof Verification**:

- Sandboxed execution of cairo-prove binary
- Timeout limits on proof verification
- Memory and CPU usage monitoring

## 8. Success Metrics

### 8.1 Functional Metrics

- **Proof Generation Success Rate**: >99% for valid signatures
- **Proof Verification Accuracy**: 100% (no false positives/negatives)
- **End-to-End Latency**: <60 seconds total
- **Proof Size**: <100KB per proof

### 8.2 Performance Metrics

- **Proof Generation Time**: <30 seconds average
- **Proof Verification Time**: <5 seconds average
- **Relay Throughput**: >10 transactions/minute
- **Memory Usage**: <1GB per component

## 9. Future Enhancements

### 9.1 Beyond MVP

**Extended Validation**:

- Multi-signature verification
- Script template validation
- Hash chain verification
- Merkle tree structure validation

**Network Integration**:

- Bitcoin P2P protocol integration
- Mining pool integration
- Mempool policy enforcement

**Scalability**:

- Batch proof generation
- Proof aggregation techniques
- Optimized verification pipelines

### 9.2 Production Readiness

**Monitoring**:

- Comprehensive logging and metrics
- Alert systems for failed verifications
- Performance dashboards

**Deployment**:

- Containerized deployment
- High-availability configurations
- Automated scaling

This PRD provides the technical foundation for implementing PONS while maintaining focus on practical, achievable goals for the initial MVP iteration.
