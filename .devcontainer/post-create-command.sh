#!/bin/sh

# Post-create command script for devcontainer setup
# This script runs after the container is created to install necessary tools

set -e

echo "=== Running post-create setup ==="

# Install global npm packages
echo "Installing Claude Code CLI..."
npm install -g @anthropic-ai/claude-code

# Verify Claude Code installation
echo "Verifying Claude Code installation..."
claude --version && echo "Claude Code is working"

# Configure git safe directory
echo "Configuring git safe directory..."
git config --global --add safe.directory /workspace

# Set up Claude configuration directory (bind mount from .git/.config/claude)
echo "Setting up Claude configuration..."
# Ensure the bind mount directory has correct permissions
# (Docker may create it with root ownership on first run)
if [ -d /home/vscode/.claude ] && [ ! -w /home/vscode/.claude ]; then
    echo "Fixing permissions on Claude config directory..."
    sudo chown -R vscode:vscode /home/vscode/.claude
fi
# Create .claude.json with default content if it doesn't exist
# On Dev Container rebuild, Claude won't use the stored credentials
# unless the .claude.json file is present and can be parsed.
# The following file is what is created by default on first run.
if [ ! -f /home/vscode/.claude/.claude.json ]; then
    echo "Creating .claude.json with default configuration..."
    cat > /home/vscode/.claude/.claude.json << 'EOF'
{
  "numStartups": 0,
  "theme": "dark",
  "preferredNotifChannel": "auto",
  "verbose": false,
  "editorMode": "normal",
  "autoCompactEnabled": true,
  "hasSeenTasksHint": false,
  "queuedCommandUpHintCount": 0,
  "diffTool": "auto",
  "customApiKeyResponses": {
    "approved": [],
    "rejected": []
  },
  "env": {},
  "tipsHistory": {},
  "memoryUsageCount": 0,
  "promptQueueUseCount": 0,
  "todoFeatureEnabled": true,
  "messageIdleNotifThresholdMs": 60000,
  "autoConnectIde": false,
  "autoInstallIdeExtension": true,
  "autocheckpointingEnabled": true,
  "checkpointingShadowRepos": [],
  "cachedStatsigGates": {}
}
EOF
fi
# Create symlink if it doesn't exist
if [ ! -L /home/vscode/.claude.json ]; then
    echo "Creating symlink ~/.claude.json -> ~/.claude/.claude.json"
    ln -sf /home/vscode/.claude/.claude.json /home/vscode/.claude.json
fi
echo "Claude configuration ready"

# Set up shell history configuration (bind mount from .git/.config/history)
echo "Setting up shell history..."
# Ensure the bind mount directory has correct permissions
if [ -d /home/vscode/.history ] && [ ! -w /home/vscode/.history ]; then
    echo "Fixing permissions on shell history directory..."
    sudo chown -R vscode:vscode /home/vscode/.history
fi
# Configure bash to use persistent history
if ! grep -q "^export HISTFILE=~/.history/bash_history" ~/.bashrc; then
    cat >> ~/.bashrc << 'EOF'
# Persistent bash history (stored in /workspace/.git/.config/history)
export HISTFILE=~/.history/bash_history
export HISTSIZE=10000
export HISTFILESIZE=20000
EOF
fi
# Configure zsh to use persistent history
if [ ! -f ~/.zshrc ] || ! grep -q "^export HISTFILE=~/.history/zsh_history" ~/.zshrc; then
    cat >> ~/.zshrc << 'EOF'
# Persistent zsh history (stored in /workspace/.git/.config/history)
export HISTFILE=~/.history/zsh_history
export HISTSIZE=10000
export SAVEHIST=20000
EOF
fi
echo "Shell history configuration ready"

echo "Setup starship prompt..."
echo "eval \"\$(starship init bash)\"" >> ~/.bashrc
echo "eval \"\$(starship init zsh)\"" >> ~/.zshrc
echo "Starship prompt setup complete"

echo "=== Post-create setup complete ==="

# EOF
