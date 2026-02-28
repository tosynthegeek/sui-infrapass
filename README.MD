# Infrapass

**Infrapass** is a decentralized service discovery and access control protocol built on Sui. Providers register their APIs on-chain, consumers purchase access with SUI, WAL, or stablecoins, and a lightweight sidecar enforces entitlements at the edge — no API keys, no manual provisioning, no code changes to your existing service.

## The Problem

API providers need a way to gate access based on on-chain payments without modifying their existing service. Infrapass acts as the enforcement layer — validating wallet-based entitlements before forwarding requests to the upstream service.

## How it works

A consumer purchases a service tier on Sui using Sui, WAL to stablecoins. The backend picks up the on-chain event, stores it, and broadcasts a cache refresh via Redis PubSub. The sidecar, running in front of the provider's API, picks this up and pre-warms its local entitlement cache. Subsequent requests are validated in under 1ms from cache.

## Full architecture

```mermaid
---
config:
  theme: base
  themeVariables:
    primaryColor: '#2a2a2a'
    primaryTextColor: '#e8e8e8'
    primaryBorderColor: '#444444'
    lineColor: '#666666'
    secondaryColor: '#333333'
    tertiaryColor: '#2a2a2a'
    background: '#1a1a1a'
    mainBkg: '#2a2a2a'
    nodeBorder: '#444444'
    clusterBkg: '#222222'
    clusterBorder: '#444444'
    titleColor: '#e8e8e8'
    edgeLabelBackground: '#2a2a2a'
    attributeBackgroundColorEven: '#2a2a2a'
    attributeBackgroundColorOdd: '#333333'
  layout: dagre
---
flowchart TB
 subgraph SUI["SUI BLOCKCHAIN"]
        SC1["Provider Registry Contract"]
        SC2["Entitlement Purchase Contract"]
  end
 subgraph SERVER["INFRAPASS BACKEND SERVER"]
        EL["Event Listener\ngRPC Checkpoint Stream"]
        EW["Event Worker\nEvent Dispatcher"]
        VA["Validator API\nPOST /validate\nPOST /record_usage"]
        REPO["Repository\nSQLx"]
  end
 subgraph INFRA["INFRASTRUCTURE"]
        DB[("TimescaleDB\nproviders\nservices\npricing_tiers\nentitlements")]
        RB[("Redis Backend\nport 6380\nPubSub Publisher")]
  end
 subgraph SIDECAR["INFRAPASS SIDECAR"]
        SUB["PubSub Subscriber\nListens to provider channel"]
        PH["Proxy Handler\nEntitlement Check\nQuota Enforcement\nRequest Forwarding"]
        RC[("Redis Cache\nentitlement keys\nquota keys")]
        LUA["Lua Script\nAtomic Quota\nDecrement"]
        AM["Auth Middleware\nNone / ApiKey\nBearerToken"]
  end
    CLI["infrapass-cli\nPurchase Entitlement"] -- submit tx --> SC2
    SC2 -- EntitlementPurchased event --> EL
    SC1 -- ProviderRegistered\nServiceCreated\nTierCreated --> EL
    EL -- EventPayload via mpsc --> EW
    EW -- persist --> REPO
    REPO -- read/write --> DB
    EW -- publish_refresh --> RB
    RB -- PubSub message\ninfrpass:provider_id:events --> SUB
    SUB -- invalidate + set_entitlement\nset_quota --> RC
    CLIENT["Client\nSui Wallet Address"] -- "HTTP request +\nX-Infrapass-Address\nX-Infrapass-Service-Id" --> AM
    AM -- authenticated --> PH
    PH -- cache hit --> RC
    PH -- cache miss POST /validate --> VA
    VA -- query valid entitlement --> REPO
    VA -- ValidateResponse --> PH
    PH -- set_entitlement\nset_quota --> RC
    PH -- invoke Lua script --> LUA
    LUA -- check + DECRBY --> RC
    PH -- "forward request +\nX-Infrapass-Validated: true\nX-Infrapass-User-Address" --> UP["Upstream Service\nProvider API"]
    PH -. async POST /record_usage .-> VA
    VA -. commit_usage DECRBY .-> REPO

     SC1:::sui
     SC2:::sui
     EL:::server
     EW:::server
     VA:::server
     REPO:::server
     DB:::infra
     RB:::infra
     SUB:::sidecar
     PH:::sidecar
     RC:::sidecar
     LUA:::sidecar
     AM:::sidecar
     UP:::upstream
     CLI:::cli
     CLIENT:::client
    classDef sui fill:#2a3535,stroke:#44ddff,color:#44ddff
    classDef server fill:#2a2535,stroke:#aa66ff,color:#aa66ff
    classDef sidecar fill:#253530,stroke:#00ff88,color:#00ff88
    classDef infra fill:#352525,stroke:#ff4455,color:#ff4455
    classDef upstream fill:#353025,stroke:#ffcc44,color:#ffcc44
    classDef client fill:#252535,stroke:#4488ff,color:#4488ff
    classDef cli fill:#2a2a2a,stroke:#555555,color:#aaaaaa
```

## Repository Structure

| Component         | Description                                                              |
| ----------------- | ------------------------------------------------------------------------ |
| infrapass-server  | gRPC event indexer, event worker, and validator HTTP API                 |
| infrapass-sidecar | Reverse proxy enforcing on-chain entitlements for provider APIs          |
| infrapass-cli     | CLI for provider registration, tier management, and entitlement purchase |
| infrapass-contracts    | Move smart contracts deployed on Sui handling provider registration, service definition, pricing tiers, entitlement minting, and on-chain access control logic |

## Infrastructure:

- **TimescaleDB** — canonical store for providers, services, tiers, and entitlements
- **Redis (backend)** — PubSub channel for entitlement refresh events
- **Redis (sidecar)** — local entitlement cache and atomic quota counters

## Quick Start (Local Development)

**1. Clone and build**

```bash
git clone https://github.com/yourusername/infrapass.git
cd infrapass
cargo build --release
```

**2. Setup Infrastructure**

```bash
docker-compose up -d
```

**3. Configure environment**

```bash
cp .env.example .env
# Edit .env with your values
```

Key variables:

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5435/infrapass
BACKEND_REDIS_URL=redis://:password@localhost:6380
GRPC_URL=https://fullnode.testnet.sui.io:443
VALIDATOR_API_KEY=your-api-key
```

**4. Run the backend server**

```bash
cargo run --bin infrapass-server
```

```bash
0.002s  INFO Starting Infrapass
0.499s  INFO Database connected and migrations applied
0.501s  INFO Validator API listening on 0.0.0.0:8088
0.502s  INFO Event listener connecting to Sui fullnode
0.891s  INFO Checkpoint stream connected
0.892s  INFO All services running
```

**5. Run the sidecar**

```bash
cargo run --bin infrapass-sidecar
```

```bash
0.002s  INFO Sidecar starting upstream=http://localhost:4000 port=8080
0.499s  INFO Listening on 0.0.0.0:8080
0.501s  INFO Subscribed channel=infrapass:0x6dc7...485d3:events
```

### Provider Setup

Providers point the sidecar at their existing service and set two environment variables. No code changes required.

```bash
UPSTREAM_URL=http://localhost:4000
VALIDATOR_API_URL=http://localhost:8088
VALIDATOR_API_KEY=your-api-key
PROVIDER_ID=0x6dc7...
REDIS_URL=redis://:password@localhost:6380
```

The sidecar proxies all traffic, forwarding two injected headers to the upstream:

```bash
X-Infrapass-Validated: true
X-Infrapass-User-Address: 0x693e... # verified Sui wallet of the caller
```

Your upstream can trust any request that carries X-Infrapass-Validated: true and reject anything that doesn't.

## Consumer Integration

Consumers add two headers to their existing requests:

```bash
# Before — direct to provider API
curl https://api.provider.com/v1/endpoint \
 -H "Authorization: Bearer your-token" \
 -d '{"key": "value"}'

# After — through Infrapass sidecar (only two headers added)

curl https://sidecar.provider.com/v1/endpoint \
 -H "Authorization: Bearer your-token" \ # unchanged, forwarded as-is
-H "X-Infrapass-Address: 0x693e..." \ # your Sui wallet address
-H "X-Infrapass-Service-Id: 0x475a..." \ # on-chain service ID
-d '{"key": "value"}' # unchanged
```

## CLI Reference

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
