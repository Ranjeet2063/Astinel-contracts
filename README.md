# Astinel Contracts

Soroban smart contracts powering the Astinel platform — a security and DevSecOps suite for the Stellar ecosystem.

## Contracts

| Contract | Description |
|---|---|
| `astinel-access-control` | Role-based access control with admin, operators, and pausing |
| `astinel-payable` | ERC-20-like token with allowance and transfer_from |
| `astinel-ttl-management` | Persistent storage with configurable time-to-live |
| `astinel-upgradeable` | Contract upgrade mechanism with wasm hash and versioning |
| `astinel-stellar-asset` | Stellar Asset Frontend (SAFE) standard: mint, burn, transfer, approve |

## Prerequisites

- Rust nightly (for Soroban)
- `wasm32-unknown-unknown` target
- Soroban CLI

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli
```

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cargo test --features testutils
```

## Deploy

Deploy contracts to a Stellar network using the Astinel backend API:

```bash
curl -X POST http://localhost:8080/v1/projects/{project_id}/contracts \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"contract_name": "astinel-access-control", "network": "testnet"}'
```

Or use the Soroban CLI directly:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/astinel_access_control.wasm \
  --network testnet
```
