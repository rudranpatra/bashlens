#!/bin/bash
set -e

# Download
GET=https://example.com/install.tar.gz
curl -fsSL "$GET" -o install.tar.gz

# Verify
sha256sum install.tar.gz

# Install
sudo tar -xzf install.tar.gz -C /usr/local/bin

# Hook shell profile
echo 'eval "$($HOME/.local/bin/app init)"' >> "$HOME/.bashrc"
