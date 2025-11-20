# 🐜 AntSol CLI – Decentralized Package Registry on Solana

AntSol is a decentralized package registry backed by on-chain metadata and IPFS content addressing. Use the CLI to publish, discover, and install packages with end-to-end verifiability.

## ✨ What it does

- 📦 Publish packages with immutable content (IPFS)
- ⛓️ Record package metadata and versions on Solana
- 🔎 Discover and inspect packages (Indexer optional)
- 🔐 Own packages by signing with your wallet
- 🧬 Version management with SemVer
- 🌐 Multi-gateway IPFS download + integrity checks

## 🚀 Install & Build

```bash
# Build
cargo build --release

# Install globally (optional)
cargo install --path .
```

## 🔧 Setup

1) Configure IPFS pinning (Pinata JWT)
```bash
export PINATA_JWT="your_pinata_jwt_token_here"
```

2) Connect a Solana wallet
```bash
antsol wallet connect ~/.config/solana/id.json
```

3) Optional env overrides for config
```bash
export ANTSOL_RPC_URL=https://api.devnet.solana.com
export ANTSOL_PROGRAM_ID=A9igkBugcujD9Nw9d97FFN4aY3qHXnJxEqCChJt8C42S
export ANTSOL_IPFS_URL=https://api.pinata.cloud
```

## 📖 Usage

### Initialize
```bash
antsol init
```
Creates `antsol.toml` manifest.

### Publish
```bash
antsol publish                 # from current directory
antsol publish --version 1.0.0 # override version
```
Process: validate → tar.gz → upload to IPFS → submit on-chain tx.

### Install
```bash
antsol install my-package@1.0.0
```
Resolves on-chain, verifies dependencies, downloads and verifies via IPFS, extracts to `antsol_packages/`.

### Info
```bash
antsol info my-package@1.0.0
```
Shows package metadata, IPFS CID, authority, and on-chain PDA.

### Search
```bash
antsol search token
```
Uses an indexer (optional). Fallback guidance provided if unavailable.

### Update
```bash
antsol update --version 1.0.1
```
Uploads new content to IPFS and records a new on-chain version.

## 🧾 Manifest (antsol.toml)
```toml
[package]
name = "my-package"
version = "1.0.0"
description = "Description"

[[dependencies]]
name = "dep-package"
version = "1.0.0"
```

## 🔐 Security & Integrity
- Wallet-signed transactions prove authorship
- PDAs `["package", name, version]` prevent collisions
- IPFS CID binds content; downloads verified locally

## ⚙️ Config file (~/.antsol/config.toml)
```toml
rpc_url = "https://api.devnet.solana.com"
ipfs_url = "https://api.pinata.cloud"
program_id = "A9igkBugcujD9Nw9d97FFN4aY3qHXnJxEqCChJt8C42S"
wallet_path = "/path/to/wallet.json"
```
Env vars override file values.

## 📜 License
MIT
