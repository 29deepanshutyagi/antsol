# AntSol Indexer v2

Blockchain indexer and REST API for the AntSol decentralized package registry.

## Overview

The indexer continuously scans the Solana blockchain for package events, parses them, and stores package metadata in PostgreSQL for fast queries. It provides a REST API for package search and discovery.

## Features

- ✅ Real-time blockchain scanning
- ✅ Event parsing and ingestion
- ✅ PostgreSQL database for package metadata
- ✅ REST API for search and discovery
- ✅ Docker deployment ready
- ✅ Configurable via environment variables

## Prerequisites

- Rust (latest stable)
- PostgreSQL 14+
- Solana RPC endpoint (devnet or mainnet)

## Setup

1. **Clone and install dependencies:**
   ```bash
   cargo build --release
   ```

2. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your credentials
   ```

3. **Required environment variables:**
   - `DATABASE_URL` - PostgreSQL connection string
   - `SOLANA_RPC_URL` - Solana RPC endpoint
   - `ANTSOL_PROGRAM_ID` - Your deployed program ID
   - `PORT` - API server port (default: 8080)

## Running

### Local Development
```bash
RUST_LOG=debug cargo run --release
```

### Docker
```bash
docker-compose up -d
```

## API Endpoints

- `GET /api/packages` - List all packages
- `GET /api/packages/:name` - Get package details
- `GET /api/search?q=term` - Search packages
- `GET /api/stats` - Registry statistics

## Architecture

```
Solana Blockchain → RPC Client → Event Parser → PostgreSQL
                                                      ↓
                                                 REST API
```

## Database Schema

- **packages** - Package metadata (name, author, description)
- **versions** - Package versions (version, IPFS CID, downloads)
- **events** - Raw blockchain events (for audit trail)
- **indexer_state** - Last processed slot (for resume capability)

## Configuration

See `.env.example` for all available configuration options.

## Deployment

### Render.com
```bash
# Uses render.yaml for configuration
git push origin main
```

### Docker
```bash
docker build -t antsol-indexer .
docker run -p 8080:8080 --env-file .env antsol-indexer
```

## Troubleshooting

**Indexer not updating:**
- Check RPC endpoint is accessible
- Verify program ID matches deployed contract
- Check database connection
- Review logs with `RUST_LOG=debug`

**Database connection issues:**
- Verify DATABASE_URL is correct
- Check PostgreSQL is running
- Ensure migrations have run

## License

MIT
