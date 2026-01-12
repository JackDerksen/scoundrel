#!/bin/bash

set -e
echo "Building Scoundrel..."
cargo build --release

# Create ~/.local/bin if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary over to local bin
cp target/release/scoundrel ~/.local/bin/
echo "✓ Installation complete!"
