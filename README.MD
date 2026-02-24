# Infrapass

**Infrapass** is a decentralized service discovery protocol built on Sui blockchain.  
Providers register services, define pricing tiers (subscription, quota-based, usage-based), and users purchase entitlements to access APIs/services with on-chain payments and usage tracking.

This repository contains:

- Backend indexer & event processor (TimescaleDB + PostgreSQL)
- CLI tool for provider & pricing management
- gRPC event listener for Sui blockchain events

## Prerequisites

- Rust (1.75+ recommended)
- PostgreSQL 16+ with TimescaleDB extension
- Sui CLI (for wallet / transaction signing)
- Docker & Docker Compose (recommended for local DB)

## Quick Start (Local Development)

1. Clone the repo

```bash
git clone https://github.com/yourusername/infrapass.git
cd infrapass
```

2. Install Dependencies

```bash
cargo build --release
```

3. Set Up Database

```bash
docker-compose up -d

# Wait for database to be ready

docker-compose logs -f timescaledb
```

## Qucik Start

1. Run the events server

```bash
cargo run --bin infrapass-server

**Output:**

2026-02-20T10:00:00Z INFO Starting Infrapass Event Listener
2026-02-20T10:00:01Z INFO Database connected
2026-02-20T10:00:01Z INFO All workers started
2026-02-20T10:00:01Z INFO Health Check: ✓ Connected | Last checkpoint: 299288730

```

## Commands

1. Register a provider

```bash
infrapass-cli provider register --metadata-uri <METADATA_URI>
```

2. Create a Service

```bash
infrapass-cli provider create-service --service-type <SERVICE_TYPE> --metadata-uri <METADATA_URI>
```

3. Update service metadata

```bash
infrapass-cli provider update-service-metadata --service-id <SERVICE_ID> --metadata-uri <METADATA_URI>
```

4. Set a service as active

```bash
infrapass-cli provider set-service-active --service-id <SERVICE_ID>
```

5. Create a new pricing tier

```bash
infrapass-cli pricing create-tier --service-id <SERVICE_ID> --name <TIER_NAME> --tier <TIER_TYPE> --price <PRICE> --coin-type <COIN_TYPE> [--duration <DAYS>] [--quota <QUOTA>]
```

6. Add tier to a service

```bash
infrapass-cli pricing add-to-service --service-id <SERVICE_ID> --tier-id <TIER_ID>
```

7. Update tier price

```bash
infrapass-cli pricing update-price --tier-id <TIER_ID> --new-price <NEW_PRICE> --coin-type <COIN_TYPE>
```

8. Deactivate a tier

```bash
infrapass-cli pricing deactivate --tier-id <TIER_ID> --coin-type <COIN_TYPE>
```

9. Reactivate a tier

```bash
infrapass-cli pricing reactivate --tier-id <TIER_ID> --coin-type <COIN_TYPE>
```

10. Remove tier from service

```bash
infrapass-cli pricing remove-from-service --tier-id <TIER_ID> --service-id <SERVICE_ID>
```

11. Purchase an entitlement

```bash
infrapass-cli payment purchase --service-id <SERVICE_ID> --tier-id <TIER_ID> --amount <AMOUNT>
```
