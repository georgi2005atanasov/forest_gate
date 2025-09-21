#!/bin/bash

# generate_ed25519_keys.sh
# This script generates Ed25519 private and public keys for JWT

# Output file names
PRIVATE_KEY="private.pem"
PUBLIC_KEY="public.pem"

# Generate private key
openssl genpkey -algorithm ed25519 -out "$PRIVATE_KEY"

# Generate public key from private key
openssl pkey -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY"

echo "✅ Keys generated:"
echo " - Private key: $PRIVATE_KEY"
echo " - Public key:  $PUBLIC_KEY"

# Optional: print them to screen
echo
echo "🔑 Private key content:"
cat "$PRIVATE_KEY"
echo
echo "🔑 Public key content:"
cat "$PUBLIC_KEY"
