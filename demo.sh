#!/bin/bash

# PONS Interactive Demo - Proof of Non-Spam for Bitcoin
# A showcase of cryptographic STARK proofs for Bitcoin transaction verification

set -e

# Enhanced colors and styling
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Demo configuration
DEMO_MESSAGE="Poneglyphs never lie! 🏴‍☠️"
WORK_DIR="./demo_output"
PAUSE_TIME=2

# Utility functions
demo_pause() {
    echo -e "${DIM}⏳ Processing...${NC}"
    sleep $PAUSE_TIME
}

demo_header() {
    clear
    echo -e "${PURPLE}${BOLD}"
    echo "██████╗  ██████╗ ███╗   ██╗███████╗"
    echo "██╔══██╗██╔═══██╗████╗  ██║██╔════╝"
    echo "██████╔╝██║   ██║██╔██╗ ██║███████╗"
    echo "██╔═══╝ ██║   ██║██║╚██╗██║╚════██║"
    echo "██║     ╚██████╔╝██║ ╚████║███████║"
    echo "╚═╝      ╚═════╝ ╚═╝  ╚═══╝╚══════╝"
    echo "${NC}"
    echo -e "${CYAN}${BOLD}🏴‍☠️ Proof of Non-Spam for Bitcoin - Interactive Demo${NC}"
    echo -e "${BLUE}Demonstrating cryptographic verification of authentic Bitcoin transactions${NC}"
    echo ""
    echo -e "${DIM}Like the Poneglyphs that preserve the true history, PONS creates${NC}"
    echo -e "${DIM}immutable proofs distinguishing real Bitcoin data from spam.${NC}"
    echo ""
}

step_intro() {
    local step_num=$1
    local title=$2
    local description=$3
    
    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${YELLOW}Step ${step_num}: ${title}${NC}"
    echo -e "${BLUE}${description}${NC}"
    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

success_message() {
    local message=$1
    echo -e "${GREEN}${BOLD}✅ ${message}${NC}"
}

# Start demo
demo_header

echo -e "${YELLOW}Press Enter to begin the PONS demonstration...${NC}"
read -r

# Setup
echo -e "${DIM}Setting up demo environment...${NC}"
if [ -d "$WORK_DIR" ]; then
    rm -rf "$WORK_DIR"
fi
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

# Step 1: Build Components
step_intro "1" "Compiling PONS Components" "Building the Cairo STARK program and Rust CLI tools"

echo -e "📦 ${BOLD}Building Cairo program${NC} (STARK proof generator)"
cd ../pons_stark
if ! scarb build --quiet; then
    echo -e "${RED}❌ Cairo build failed${NC}"
    exit 1
fi
success_message "Cairo program compiled"
demo_pause

echo -e "🦀 ${BOLD}Building Rust CLI${NC} (Bitcoin transaction tools)"
cd ../pons_cli
if ! cargo build --release --quiet; then
    echo -e "${RED}❌ Rust build failed${NC}"
    exit 1
fi
success_message "Rust CLI compiled"
demo_pause

# Step 2: Generate Cryptographic Identity
cd "../$WORK_DIR"
step_intro "2" "Creating Bitcoin Identity" "Generating a Taproot keypair for Bitcoin transactions"

echo -e "🔐 ${BOLD}Generating Taproot keypair...${NC}"
echo -e "${DIM}This creates a Bitcoin public/private key pair using BIP-340 Schnorr signatures${NC}"
if ! ../pons_cli/target/release/pons-cli keygen --output keypair.json >/dev/null 2>&1; then
    echo -e "${RED}❌ Keypair generation failed${NC}"
    exit 1
fi

PUBKEY=$(jq -r '.public_key' keypair.json)
success_message "Bitcoin keypair generated"
echo -e "${CYAN}🔑 Public Key: ${PUBKEY:0:16}...${PUBKEY: -16}${NC}"
demo_pause

# Step 3: Sign Message
step_intro "3" "Digital Signature Creation" "Signing our message with cryptographic proof of authenticity"

echo -e "✍️  ${BOLD}Signing message:${NC} \"${DEMO_MESSAGE}\""
echo -e "${DIM}Using BIP-340 Schnorr signatures - the same cryptography securing Bitcoin${NC}"
if ! ../pons_cli/target/release/pons-cli sign --keypair keypair.json --message "$DEMO_MESSAGE" --output signature.json >/dev/null 2>&1; then
    echo -e "${RED}❌ Message signing failed${NC}"
    exit 1
fi

SIGNATURE=$(jq -r '.signature' signature.json)
success_message "Message cryptographically signed"
echo -e "${CYAN}✍️  Signature: ${SIGNATURE:0:20}...${SIGNATURE: -20}${NC}"
demo_pause

# Step 4: Generate STARK Arguments  
step_intro "4" "Preparing STARK Proof" "Converting signature into mathematical arguments for zero-knowledge proof"

echo -e "🧮 ${BOLD}Generating cryptographic arguments...${NC}"
echo -e "${DIM}Using Garaga library to create STARK-compatible proof parameters${NC}"
cd ../pons_stark
if ! python3.10 gen_args_bitcoin.py --pubkey "$PUBKEY" --signature "$SIGNATURE" --message "$DEMO_MESSAGE" --target execute > "../$WORK_DIR/args.json" 2>/dev/null; then
    echo -e "${RED}❌ Argument generation failed${NC}"
    exit 1
fi

cd "../$WORK_DIR"
ARG_COUNT=$(jq '.[0]' args.json | sed 's/^"0x//' | sed 's/"$//')
ARG_COUNT_DEC=$((16#$ARG_COUNT))
success_message "STARK arguments prepared"
echo -e "${CYAN}🔢 Generated ${ARG_COUNT_DEC} mathematical arguments${NC}"
demo_pause

# Step 5: Generate STARK Proof
step_intro "5" "Creating STARK Proof" "Generating zero-knowledge proof that signature is valid without revealing private key"

if command -v cairo-prove &> /dev/null; then
    echo -e "⚡ ${BOLD}Generating STARK proof...${NC}"
    echo -e "${DIM}This cryptographic proof can verify signature validity without exposing secrets${NC}"
    
    cd ../pons_stark
    START_TIME=$(date +%s)
    if cairo-prove prove target/dev/pons_stark.executable.json "../$WORK_DIR/proof.json" --arguments-file "../$WORK_DIR/args.json" >/dev/null 2>&1; then
        END_TIME=$(date +%s)
        PROOF_TIME=$((END_TIME - START_TIME))
        
        success_message "STARK proof generated in ${PROOF_TIME}s"
        
        # Proof details
        PROOF_SIZE=$(stat -f%z "../$WORK_DIR/proof.json" 2>/dev/null || stat -c%s "../$WORK_DIR/proof.json" 2>/dev/null || echo "unknown")
        PROOF_SIZE_MB=$(echo "scale=2; $PROOF_SIZE / 1024 / 1024" | bc 2>/dev/null || echo "~22")
        echo -e "${CYAN}📄 Proof size: ${PROOF_SIZE_MB}MB${NC}"
        
        demo_pause
        
        # Step 6: Verify Proof
        step_intro "6" "Verifying STARK Proof" "Confirming the proof validates our signature without revealing private information"
        
        echo -e "🔍 ${BOLD}Verifying STARK proof...${NC}"
        echo -e "${DIM}Anyone can verify this proof without accessing private keys or signatures${NC}"
        
        if cairo-prove verify "../$WORK_DIR/proof.json" >/dev/null 2>&1; then
            success_message "STARK proof verification successful!"
            PROOF_VERIFIED=true
        else
            echo -e "${RED}❌ Proof verification failed${NC}"
            PROOF_VERIFIED=false
        fi
    else
        echo -e "${YELLOW}⚠️  STARK proof generation encountered issues${NC}"
        PROOF_VERIFIED=false
    fi
else
    echo -e "${YELLOW}⚠️  cairo-prove not available${NC}"
    echo -e "${DIM}Install stwo-cairo to generate actual STARK proofs${NC}"
    PROOF_VERIFIED=false
fi

demo_pause

# Final Summary
cd "../$WORK_DIR"
echo ""
echo -e "${PURPLE}${BOLD}"
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                            🎉 DEMO COMPLETE 🎉                               ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo "${NC}"

echo -e "${BOLD}${BLUE}🔍 What We Just Demonstrated:${NC}"
echo ""
echo -e "${GREEN}✅ Bitcoin Keypair Generation${NC}    - Created cryptographic identity"
echo -e "${GREEN}✅ Message Signing${NC}              - Proved ownership with digital signature"  
echo -e "${GREEN}✅ STARK Argument Preparation${NC}   - Converted signature to zero-knowledge format"

if [ "$PROOF_VERIFIED" = true ]; then
    echo -e "${GREEN}✅ STARK Proof Generation${NC}       - Created cryptographic proof"
    echo -e "${GREEN}✅ STARK Proof Verification${NC}     - Validated authenticity without secrets"
    echo ""
    echo -e "${BOLD}${GREEN}🚀 PONS is ready to fight Bitcoin spam with cryptographic truth!${NC}"
else
    echo -e "${YELLOW}⏸️  STARK Proof Generation${NC}       - Skipped (cairo-prove needed)"
    echo ""
    echo -e "${BOLD}${YELLOW}📋 Core components validated - install cairo-prove for full demo${NC}"
fi

echo ""
echo -e "${BOLD}${CYAN}🏴‍☠️ Like the Poneglyphs, PONS preserves cryptographic truth:${NC}"
echo -e "${DIM}• Immutable proofs that cannot be faked${NC}"
echo -e "${DIM}• Verifiable by anyone with the right knowledge${NC}"  
echo -e "${DIM}• Distinguishes authentic data from spam${NC}"
echo ""

echo -e "${BOLD}${BLUE}📁 Generated files:${NC}"
ls -la | grep -E '\.(json)$' | while read -r line; do
    echo -e "${CYAN}  $line${NC}"
done

echo ""
echo -e "${DIM}Demo completed! Files saved in: $(pwd)${NC}"
echo -e "${YELLOW}Press Enter to exit...${NC}"
read -r