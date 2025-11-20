#!/bin/bash
# AntSol CLI Quick Start Setup Script
# This script helps you set up everything needed to use AntSol

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Symbols
CHECK="${GREEN}✓${NC}"
CROSS="${RED}✗${NC}"
ARROW="${CYAN}→${NC}"

echo -e "${CYAN}"
echo "╔══════════════════════════════════════════════╗"
echo "║   🐜 AntSol - Decentralized Registry Setup  ║"
echo "║   On-Chain Package Management on Solana      ║"
echo "║   Immutable Storage via IPFS                 ║"
echo "╚══════════════════════════════════════════════╝"
echo -e "${NC}\n"

command_exists() { command -v "$1" >/dev/null 2>&1; }
print_step() { echo -e "\n${BLUE}═══ $1 ═══${NC}"; }
print_info() { echo -e "${ARROW} $1"; }
print_success() { echo -e "${CHECK} $1"; }
print_error() { echo -e "${CROSS} $1"; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }

print_step "Checking Prerequisites"

if command_exists rustc; then
  RUST_VERSION=$(rustc --version | cut -d' ' -f2)
  print_success "Rust installed: v$RUST_VERSION"
else
  print_error "Rust not found"
  print_info "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source $HOME/.cargo/env
  print_success "Rust installed"
fi

if command_exists solana; then
  SOLANA_VERSION=$(solana --version | cut -d' ' -f2)
  print_success "Solana CLI installed: v$SOLANA_VERSION"
else
  print_error "Solana CLI not found"
  print_info "Installing Solana CLI..."
  sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
  export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
  print_success "Solana CLI installed"
fi

print_step "Setting Up Solana Wallet"
WALLET_PATH="$HOME/.config/solana/antsol-keypair.json"
if [ -f "$WALLET_PATH" ]; then
  print_success "Wallet already exists at $WALLET_PATH"
else
  print_info "Creating new Solana wallet..."
  mkdir -p "$HOME/.config/solana"
  solana-keygen new --no-bip39-passphrase --outfile "$WALLET_PATH"
  chmod 600 "$WALLET_PATH"
  print_success "Wallet created at $WALLET_PATH"
fi
WALLET_ADDRESS=$(solana-keygen pubkey "$WALLET_PATH")
print_info "Your wallet address: ${GREEN}$WALLET_ADDRESS${NC}"

print_step "Configuring Solana Network"
print_info "Setting network to devnet..."
solana config set --url https://api.devnet.solana.com --keypair "$WALLET_PATH" >/dev/null 2>&1
print_success "Network configured to devnet"

print_step "Requesting Devnet SOL"
BALANCE=$(solana balance "$WALLET_ADDRESS" 2>/dev/null | cut -d' ' -f1)
print_info "Current balance: $BALANCE SOL"
if (( $(echo "$BALANCE < 1" | bc -l) )); then
  print_info "Requesting airdrop (2 SOL)..."
  solana airdrop 2 "$WALLET_ADDRESS" >/dev/null 2>&1 || print_warning "Airdrop failed. Try manually: solana airdrop 2"
  sleep 3
  NEW_BALANCE=$(solana balance "$WALLET_ADDRESS" 2>/dev/null | cut -d' ' -f1)
  print_success "New balance: $NEW_BALANCE SOL"
else
  print_success "Sufficient balance"
fi

print_step "Configuring IPFS Storage (Pinata)"
if [ -z "$PINATA_JWT" ]; then
  print_warning "PINATA_JWT environment variable not set"
  echo ""
  print_info "To publish packages, you need a Pinata JWT token:"
  echo -e "  1. Sign up at ${BLUE}https://pinata.cloud/${NC}"
  echo "  2. Create an API key with pinning permissions"
  echo "  3. Copy the JWT token"
  echo ""
  read -p "Enter your Pinata JWT token (or press Enter to skip): " JWT_INPUT
  if [ -n "$JWT_INPUT" ]; then
    export PINATA_JWT="$JWT_INPUT"
    SHELL_PROFILE="$HOME/.bashrc"
    [ -f "$HOME/.zshrc" ] && SHELL_PROFILE="$HOME/.zshrc"
    {
      echo ""
      echo "# AntSol Pinata JWT"
      echo "export PINATA_JWT=\"$JWT_INPUT\""
    } >> "$SHELL_PROFILE"
    print_success "PINATA_JWT saved to $SHELL_PROFILE"
  else
    print_warning "Skipping Pinata setup. You can set it later with: export PINATA_JWT=\"your_token\""
  fi
else
  print_success "PINATA_JWT already configured"
fi

print_step "Building AntSol CLI"
if [ ! -f "Cargo.toml" ]; then
  print_error "Cargo.toml not found. Run from the antsol-cli directory"
  exit 1
fi
print_info "Building CLI (this may take a few minutes)..."
cargo build --release
if [ -f "target/release/antsol" ]; then
  print_success "CLI built successfully"
  read -p "Install CLI globally? (y/n): " INSTALL_GLOBAL
  if [ "$INSTALL_GLOBAL" = "y" ]; then
    cargo install --path .
    print_success "CLI installed globally"
  else
    print_info "You can run the CLI with: ./target/release/antsol"
  fi
else
  print_error "Build failed"
  exit 1
fi

print_step "Connecting Wallet to AntSol"
if command_exists antsol; then
  antsol wallet connect "$WALLET_PATH"
  print_success "Wallet connected"
else
  print_warning "Run: ./target/release/antsol wallet connect $WALLET_PATH"
fi

print_step "Setup Complete! 🎉"
print_success "AntSol CLI is ready to use"
echo ""
echo -e "${CYAN}Quick Reference:${NC}"
echo -e "  ${GREEN}antsol init${NC}                    - Initialize new package"
echo -e "  ${GREEN}antsol publish${NC}                 - Publish package"
echo -e "  ${GREEN}antsol install <pkg>@<ver>${NC}   - Install package"
echo -e "  ${GREEN}antsol search <query>${NC}         - Search packages"
echo -e "  ${GREEN}antsol info <package>${NC}         - View package info"
echo -e "  ${GREEN}antsol wallet show${NC}            - Show wallet info"
echo ""
echo -e "${CYAN}Your Configuration:${NC}"
echo -e "  Wallet: ${GREEN}$WALLET_ADDRESS${NC}"
echo -e "  Network: ${GREEN}Devnet${NC}"
echo -e "  Program: ${GREEN}A9igkBugcujD9Nw9d97FFN4aY3qHXnJxEqCChJt8C42S${NC}"
echo ""
