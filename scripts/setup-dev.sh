#!/bin/sh
# Setup script for development environment

echo "Setting up development environment..."

# Configure git hooks
git config core.hooksPath .githooks
echo "✓ Git hooks configured"

# Check for required tools
if ! command -v rustfmt &> /dev/null; then
    echo "Installing rustfmt..."
    rustup component add rustfmt
fi

if ! command -v cargo-clippy &> /dev/null; then
    echo "Installing clippy..."
    rustup component add clippy
fi

echo "✓ Required tools installed"
echo ""
echo "✅ Development environment setup complete!"
