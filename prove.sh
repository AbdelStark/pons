#!/bin/bash

set -e

EXECUTABLE_NAME=pons
EXECUTABLE_FILE=target/dev/$EXECUTABLE_NAME.executable.json
PROOF_FILE=./pons_proof.json

# Build the Cairo program
scarb build

# Prove the program
cairo-prove prove \
  $EXECUTABLE_FILE \
  $PROOF_FILE \
  --arguments 10

# Verify the proof
cairo-prove verify $PROOF_FILE