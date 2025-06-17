#!/bin/bash

# Git Workers Setup Script

echo "Setting up Git Workers..."

# Build the project
echo "Building the project..."
cargo build --release

# Get the absolute path to the binary
BINARY_PATH="$(pwd)/target/release/gw"
SHELL_FUNCTION_PATH="$(pwd)/shell/gw.sh"

echo "Binary built at: $BINARY_PATH"

# Detect shell
if [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_CONFIG="$HOME/.zshrc"
    echo "Detected zsh shell"
elif [[ "$SHELL" == *"bash"* ]]; then
    SHELL_CONFIG="$HOME/.bashrc"
    echo "Detected bash shell"
else
    echo "Unknown shell: $SHELL"
    echo "Please manually add the following to your shell configuration:"
    echo ""
    echo "# Git Workers"
    echo "export PATH=\"\$PATH:$(pwd)/target/release\""
    echo "source $(pwd)/shell/gw.sh"
    exit 1
fi

# Check if already configured
if grep -q "git-workers" "$SHELL_CONFIG" 2>/dev/null; then
    echo "Git Workers already configured in $SHELL_CONFIG"
else
    echo "Adding Git Workers to $SHELL_CONFIG..."
    
    cat >> "$SHELL_CONFIG" << EOF

# Git Workers
export PATH="\$PATH:$(pwd)/target/release"
source $(pwd)/shell/gw.sh
EOF

    echo "Configuration added to $SHELL_CONFIG"
fi

echo ""
echo "Setup complete!"
echo ""
echo "To activate, run:"
echo "  source $SHELL_CONFIG"
echo ""
echo "Or restart your terminal."
echo ""
echo "Then test with:"
echo "  gw"