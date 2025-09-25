#!/bin/bash

# Generate EC private key (SEC1, P-256)
openssl ecparam -name prime256v1 -genkey -noout -out ec_private_sec1.pem

# Convert SEC1 -> PKCS#8 (this is what jsonwebtoken expects)
openssl pkcs8 -topk8 -nocrypt -in ec_private_sec1.pem -out ec_private_pkcs8.pem

# Extract public key (SPKI PEM)
openssl ec -in ec_private_sec1.pem -pubout -out ec_public.pem
