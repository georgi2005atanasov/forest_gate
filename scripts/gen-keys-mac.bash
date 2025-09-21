#!/bin/bash

PRIVATE_KEY="private.pem"
PUBLIC_KEY="public.pem"

if openssl genpkey -algorithm ed25519 -out "$PRIVATE_KEY" 2>/dev/null; then
    echo "✅ Generated Ed25519 keys"
    openssl pkey -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY"
else
    echo "⚠️ Ed25519 not supported on this OpenSSL. Falling back to ES256..."
    openssl ecparam -name prime256v1 -genkey -noout -out "$PRIVATE_KEY"
    openssl ec -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY"
fi

echo "✅ Keys generated:"
echo " - Private key: $PRIVATE_KEY"
echo " - Public key:  $PUBLIC_KEY"
