#!/usr/bin/env python3.10

"""
PONS Bitcoin Signature Argument Generator

Generates proper Garaga arguments for Bitcoin BIP-340 Schnorr signature verification.

Usage:
    python gen_args_bitcoin.py --pubkey <hex> --signature <hex> --message <str> [--target <cairo-run|execute>]
"""

import sys
import json
import hashlib

# Import garaga - required for real signature verification
try:
    from garaga.definitions import CurveID, CURVES  
    from garaga.starknet.tests_and_calldata_generators.signatures import SchnorrSignature
except ImportError as e:
    print(f"Error: Garaga is required but not available: {e}", file=sys.stderr)
    print("Install Garaga with: python3.10 -m pip install garaga", file=sys.stderr)
    sys.exit(1)

TWO128 = 2**128

def to_u256(value: int) -> list[int]:
    """Convert integer to u256 representation (low, high)"""
    return [value % TWO128, value // TWO128]

def hash_challenge(rx: int, px: int, msg_hash: int) -> int:
    """Compute BIP-340 challenge hash"""
    tagged_hash = hashlib.sha256("BIP0340/challenge".encode()).digest()
    input_data = (
        tagged_hash + 
        tagged_hash + 
        rx.to_bytes(32, "big") + 
        px.to_bytes(32, "big") + 
        msg_hash.to_bytes(32, "big")  
    )
    return int.from_bytes(hashlib.sha256(input_data).digest(), "big")

def derive_point_from_x(x: int, is_even: bool = True) -> tuple[int, int]:
    """
    Derive the EC point (x, y) from an x-coordinate on the secp256k1 curve.
    """
    p = CURVES[CurveID.SECP256K1.value].p
    a = CURVES[CurveID.SECP256K1.value].a  
    b = CURVES[CurveID.SECP256K1.value].b
    
    # Calculate y^2 = x^3 + ax + b mod p
    y_squared = (pow(x, 3, p) + a * x + b) % p
    
    # Compute modular square root of y^2 mod p
    y = pow(y_squared, (p + 1) // 4, p)
    
    # Select the correct y based on its parity (even/odd)
    if is_even != (y % 2 == 0):
        y = p - y
    
    return (x, y)

def handle_bitcoin_signature(pubkey_hex: str, signature_hex: str, message: str) -> list:
    """
    Generate arguments for the Cairo program from Bitcoin signature data.
    """
    # Parse public key
    if pubkey_hex.startswith('0x'):
        pubkey_hex = pubkey_hex[2:]
    px = int(pubkey_hex, 16)
    _, py = derive_point_from_x(px, is_even=True)
    
    # Parse signature
    if signature_hex.startswith('0x'):
        signature_hex = signature_hex[2:]
    
    if len(signature_hex) != 128:
        raise ValueError(f"Invalid signature length: expected 128 hex chars, got {len(signature_hex)}")
    
    sig_bytes = bytes.fromhex(signature_hex)
    rx = int.from_bytes(sig_bytes[:32], "big")
    s = int.from_bytes(sig_bytes[32:], "big")
    
    # Hash the message
    msg_hash = int.from_bytes(hashlib.sha256(message.encode()).digest(), "big")
    
    # Compute BIP-340 challenge
    n = CURVES[CurveID.SECP256K1.value].n
    
    e = hash_challenge(rx, px, msg_hash) % n
    
    print(f"Generated arguments for:", file=sys.stderr)
    print(f"  Public key: {pubkey_hex}", file=sys.stderr)
    print(f"  Message: '{message}'", file=sys.stderr)
    print(f"  Message hash: {hex(msg_hash)}", file=sys.stderr)
    print(f"  Signature R: {hex(rx)}", file=sys.stderr)
    print(f"  Signature S: {hex(s)}", file=sys.stderr)
    print(f"  Challenge E: {hex(e)}", file=sys.stderr)
    
    # Create real Garaga signature with hints
    signature = SchnorrSignature(
        rx,
        s, 
        e,
        px,
        py,
        curve_id=CurveID.SECP256K1
    )
    
    return [
        *to_u256(msg_hash),
        *signature.serialize_with_hints(),
    ]

def generate_args(pubkey: str, signature: str, message: str, target: str) -> list:
    """Generate arguments in the format expected by cairo-prove"""
    bitcoin_args = handle_bitcoin_signature(pubkey, signature, message)
    
    # Format as single event
    res = [1] + bitcoin_args  # 1 event followed by the signature data
    
    print(f"  Total args: {len(res)}", file=sys.stderr)
    
    if target == "cairo-run":
        return [res]
    elif target == "execute":
        return [hex(len(res))] + list(map(hex, res))
    else:
        raise NotImplementedError(target)

def main():
    if len(sys.argv) < 7:
        print("Usage: python gen_args_bitcoin.py --pubkey <hex> --signature <hex> --message <str> [--target <cairo-run|execute>]")
        print("")
        print("Example:")
        print("  python gen_args_bitcoin.py --pubkey a12861110c4cc57ead491a1d4b4e0b50614da388c216208419cfc1cfc39efb5a \\")
        print("                             --signature f387a124...00e1a1cb \\")
        print("                             --message 'Hello PONS!'")
        sys.exit(1)
    
    # Parse arguments
    args_dict = {}
    i = 1
    while i < len(sys.argv):
        if sys.argv[i].startswith('--'):
            key = sys.argv[i][2:]
            if i + 1 < len(sys.argv) and not sys.argv[i + 1].startswith('--'):
                args_dict[key] = sys.argv[i + 1]
                i += 2
            else:
                args_dict[key] = True
                i += 1
        else:
            i += 1
    
    # Validate required arguments
    required = ['pubkey', 'signature', 'message']
    for req in required:
        if req not in args_dict:
            print(f"Error: Missing required argument --{req}")
            sys.exit(1)
    
    target = args_dict.get('target', 'execute')
    
    try:
        args = generate_args(
            args_dict['pubkey'],
            args_dict['signature'], 
            args_dict['message'],
            target
        )
        print(json.dumps(args, indent=2))
    except Exception as e:
        print(f"Error generating arguments: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()