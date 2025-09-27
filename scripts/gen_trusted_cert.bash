#!/bin/bash

# install once
brew install mkcert nss
mkcert -install

# make ssl folder in your project root
mkdir -p ssl

# make ECDSA cert + key for localhost
mkcert -ecdsa -cert-file ssl/server.crt -key-file ssl/server.key localhost 127.0.0.1 ::1
