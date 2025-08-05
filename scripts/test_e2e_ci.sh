#!/bin/bash

# PONS End-to-End Integration Test - CI Optimized
# Enhanced debugging version for continuous integration environments
# Provides detailed logging and error diagnostics

set -e
set -x  # Enable verbose debugging output

# Test configuration
TEST_MESSAGE="Hello PONS E2E Test!"
WORK_DIR="./e2e_test_output"

# Logging functions for CI debugging
log_info() {
    echo "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

log_debug() {
    echo "[DEBUG] $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

step_header() {
    local step_num=$1
    local title=$2
    local description=$3
    
    echo "=============================================================================="
    echo "STEP ${step_num}: ${title}"
    echo "Description: ${description}"
    echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "=============================================================================="
}

# System environment debugging
log_info "Starting PONS E2E Integration Test - CI Mode"
log_info "Current working directory: $(pwd)"
log_info "User: $(whoami)"
log_info "Shell: $SHELL"
log_info "PATH: $PATH"

# Environment information
log_debug "Operating System: $(uname -a)"
log_debug "Available memory: $(free -h 2>/dev/null || vm_stat 2>/dev/null || echo 'Memory info not available')"
log_debug "Disk space: $(df -h . | tail -1)"

# Check for required tools
log_info "Checking for required tools..."
which scarb && log_info "scarb found: $(scarb --version)" || log_error "scarb not found"
which cargo && log_info "cargo found: $(cargo --version)" || log_error "cargo not found"
which jq && log_info "jq found: $(jq --version)" || log_error "jq not found"
which python3.10 && log_info "python3.10 found: $(python3.10 --version)" || log_error "python3.10 not found"

# Environment cleanup
if [ -d "$WORK_DIR" ]; then
    log_info "Cleaning previous test output directory: $WORK_DIR"
    rm -rf "$WORK_DIR"
    log_debug "Previous test output cleaned"
fi

log_info "Creating work directory: $WORK_DIR"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"
log_info "Working directory created and changed to: $(pwd)"

step_header "1" "Build Cairo Components" "Compiling STARK proof generator with detailed output"

log_info "Building Cairo program..."
log_debug "Changing to pons_stark directory"
cd ../pons_stark

log_debug "Current directory: $(pwd)"
log_debug "Contents of pons_stark directory:"
ls -la

log_info "Running scarb build..."
if ! scarb build; then
    log_error "Cairo build failed"
    log_debug "scarb build output was captured above"
    log_debug "Current directory contents after failed build:"
    ls -la
    exit 1
fi

log_info "Cairo program compiled successfully"
log_debug "Build artifacts:"
ls -la target/ || log_debug "No target directory found"

step_header "2" "Build Rust CLI" "Compiling Bitcoin transaction tools with detailed output"

log_info "Building Rust CLI..."
log_debug "Changing to pons_cli directory"
cd ../pons_cli

log_debug "Current directory: $(pwd)"
log_debug "Contents of pons_cli directory:"
ls -la

log_info "Running cargo build --release..."
if ! cargo build --release; then
    log_error "Rust build failed"
    log_debug "cargo build output was captured above"
    log_debug "Current directory contents after failed build:"
    ls -la
    exit 1
fi

log_info "Rust CLI compiled successfully"
log_debug "Binary location:"
ls -la target/release/pons-cli || log_debug "Binary not found at expected location"

step_header "3" "Generate Bitcoin Identity" "Creating Taproot keypair with detailed logging"

log_info "Generating Taproot keypair..."
log_debug "Changing back to work directory"
cd "../$WORK_DIR"
log_debug "Current directory: $(pwd)"

log_info "Running pons-cli keygen..."
log_debug "Command: ../pons_cli/target/release/pons-cli keygen --output keypair.json"

if ! ../pons_cli/target/release/pons-cli keygen --output keypair.json; then
    log_error "Keypair generation failed"
    log_debug "Exit code: $?"
    log_debug "Current directory contents:"
    ls -la
    exit 1
fi

log_info "Keypair generation completed"
log_debug "Keypair file created:"
ls -la keypair.json

if [ ! -f "keypair.json" ]; then
    log_error "keypair.json file not found after generation"
    exit 1
fi

PUBKEY=$(jq -r '.public_key' keypair.json)
if [ -z "$PUBKEY" ] || [ "$PUBKEY" = "null" ]; then
    log_error "Failed to extract public key from keypair.json"
    log_debug "Contents of keypair.json:"
    cat keypair.json
    exit 1
fi

log_info "Bitcoin keypair created successfully"
log_info "Public Key: ${PUBKEY:0:16}...${PUBKEY: -16}"
log_debug "Full public key: $PUBKEY"

step_header "4" "Create Digital Signature" "Signing message with detailed logging"

log_info "Signing message: '$TEST_MESSAGE'"
log_debug "Command: ../pons_cli/target/release/pons-cli sign --keypair keypair.json --message \"$TEST_MESSAGE\" --output signature.json"

if ! ../pons_cli/target/release/pons-cli sign --keypair keypair.json --message "$TEST_MESSAGE" --output signature.json; then
    log_error "Message signing failed"
    log_debug "Exit code: $?"
    log_debug "Current directory contents:"
    ls -la
    exit 1
fi

log_info "Message signing completed"
log_debug "Signature file created:"
ls -la signature.json

if [ ! -f "signature.json" ]; then
    log_error "signature.json file not found after signing"
    exit 1
fi

SIGNATURE=$(jq -r '.signature' signature.json)
if [ -z "$SIGNATURE" ] || [ "$SIGNATURE" = "null" ]; then
    log_error "Failed to extract signature from signature.json"
    log_debug "Contents of signature.json:"
    cat signature.json
    exit 1
fi

log_info "Message cryptographically signed"
log_info "Signature: ${SIGNATURE:0:20}...${SIGNATURE: -20}"
log_debug "Full signature: $SIGNATURE"

step_header "5" "Prepare STARK Arguments" "Converting signature to STARK format with detailed logging"

log_info "Generating cryptographic arguments..."
log_debug "Changing to pons_stark directory"
cd ../pons_stark

log_debug "Current directory: $(pwd)"
log_debug "Python script exists:"
ls -la gen_args_bitcoin.py

log_info "Running gen_args_bitcoin.py..."
log_debug "Command: python3.10 gen_args_bitcoin.py --pubkey \"$PUBKEY\" --signature \"$SIGNATURE\" --message \"$TEST_MESSAGE\" --target execute"

if ! python3.10 gen_args_bitcoin.py --pubkey "$PUBKEY" --signature "$SIGNATURE" --message "$TEST_MESSAGE" --target execute > "../$WORK_DIR/args.json"; then
    log_error "Argument generation failed"
    log_debug "Exit code: $?"
    log_debug "Python script output (if any) should be visible above"
    exit 1
fi

log_info "STARK argument generation completed"
log_debug "Changing back to work directory"
cd "../$WORK_DIR"

log_debug "Arguments file created:"
ls -la args.json

if [ ! -f "args.json" ]; then
    log_error "args.json file not found after generation"
    exit 1
fi

log_debug "Contents of args.json (first 500 chars):"
head -c 500 args.json

ARG_COUNT=$(jq '.[0]' args.json | sed 's/^"0x//' | sed 's/"$//')
if [ -z "$ARG_COUNT" ]; then
    log_error "Failed to extract argument count from args.json"
    log_debug "First element of args.json:"
    jq '.[0]' args.json
    exit 1
fi

ARG_COUNT_DEC=$((16#$ARG_COUNT))
log_info "STARK arguments prepared successfully"
log_info "Generated $ARG_COUNT_DEC mathematical arguments"

step_header "6" "Generate STARK Proof" "Creating zero-knowledge proof with detailed logging"

if command -v cairo-prove &> /dev/null; then
    log_info "cairo-prove found, generating STARK proof..."
    log_debug "cairo-prove version: $(cairo-prove --version 2>&1 || echo 'Version not available')"
    
    cd ../pons_stark
    log_debug "Current directory: $(pwd)"
    log_debug "Executable file exists:"
    ls -la target/dev/pons_stark.executable.json
    
    if [ ! -f "target/dev/pons_stark.executable.json" ]; then
        log_error "Cairo executable not found at expected location"
        log_debug "Contents of target/dev/:"
        ls -la target/dev/ || log_debug "target/dev/ directory does not exist"
        exit 1
    fi
    
    log_info "Starting STARK proof generation..."
    START_TIME=$(date +%s)
    log_debug "Start time: $START_TIME"
    log_debug "Command: cairo-prove prove target/dev/pons_stark.executable.json \"../$WORK_DIR/proof.json\" --arguments-file \"../$WORK_DIR/args.json\""
    
    if cairo-prove prove target/dev/pons_stark.executable.json "../$WORK_DIR/proof.json" --arguments-file "../$WORK_DIR/args.json"; then
        END_TIME=$(date +%s)
        PROOF_TIME=$((END_TIME - START_TIME))
        log_info "STARK proof generated successfully in ${PROOF_TIME}s"
        
        cd "../$WORK_DIR"
        log_debug "Proof file created:"
        ls -la proof.json
        
        PROOF_SIZE=$(stat -f%z "proof.json" 2>/dev/null || stat -c%s "proof.json" 2>/dev/null || echo "0")
        if [ "$PROOF_SIZE" != "0" ]; then
            PROOF_SIZE_MB=$(echo "scale=2; $PROOF_SIZE / 1024 / 1024" | bc 2>/dev/null || echo "unknown")
            log_info "Proof size: ${PROOF_SIZE_MB}MB (${PROOF_SIZE} bytes)"
        else
            log_debug "Could not determine proof file size"
        fi
        
        step_header "7" "Verify STARK Proof" "Confirming proof validates signature with detailed logging"
        
        log_info "Verifying STARK proof..."
        cd ../pons_stark
        log_debug "Command: cairo-prove verify \"../$WORK_DIR/proof.json\""
        
        if cairo-prove verify "../$WORK_DIR/proof.json"; then
            log_info "STARK proof verification successful!"
            PROOF_VERIFIED=true
        else
            log_error "STARK proof verification failed"
            log_debug "Verification exit code: $?"
            exit 1
        fi
    else
        log_error "STARK proof generation failed"
        log_debug "Proof generation exit code: $?"
        log_debug "This may be due to environment or dependency issues"
        log_debug "Current directory contents:"
        ls -la
        exit 1
    fi
else
    log_error "cairo-prove not available in PATH"
    log_debug "Available commands in PATH:"
    echo "$PATH" | tr ':' '\n' | while read dir; do
        [ -d "$dir" ] && ls "$dir" | grep -i cairo || true
    done
    log_error "Install stwo-cairo for complete STARK proof functionality"
    exit 1
fi

# Final results
cd "../$WORK_DIR"
echo "=============================================================================="
echo "TEST COMPLETE"
echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=============================================================================="

log_info "Component Validation Results:"
log_info "✓ Cairo Program Compilation - STARK proof system ready"
log_info "✓ Rust CLI Compilation - Bitcoin tools operational"
log_info "✓ Taproot Keypair Generation - Cryptographic identity created"
log_info "✓ BIP-340 Signature Creation - Message authenticity proven"
log_info "✓ STARK Argument Generation - Cairo parameters prepared"

if [ "${PROOF_VERIFIED:-false}" = "true" ]; then
    log_info "✓ STARK Proof Generation - Cryptographic proof created"
    log_info "✓ STARK Proof Verification - Cryptographic proof verified"
    log_info "FULL END-TO-END TEST PASSED!"
else
    log_error "STARK Proof Generation/Verification - Failed"
    log_info "CORE COMPONENTS VALIDATED BUT STARK PROOF FAILED!"
fi

log_info "Generated artifacts in $(pwd):"
ls -la | grep -E '\.(json)$' | while read -r line; do
    log_info "  $line"
done

log_debug "Final directory contents:"
ls -la

log_info "Test completed successfully in directory: $(pwd)"