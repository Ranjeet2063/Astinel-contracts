# Astinel Contracts

Soroban smart contract suite for the Astinel security platform.

[![CI](https://github.com/Astinel-Org/Astinel-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Astinel-Org/Astinel-contracts/actions/workflows/ci.yml)

## Contracts

| Contract | Public Functions | Tests | Custom Types |
|---|---|---|---|
| `astinel-access-control` | 9 | 3 | 0 |
| `astinel-payable` | 8 | 3 | 1 |
| `astinel-ttl-management` | 7 | 2 | 1 |
| `astinel-upgradeable` | 8 | 3 | 0 |
| `astinel-stellar-asset` | 14 | 3 | 2 |

All contracts are self-contained with no cross-contract calls. Each initializes with an admin address and uses Soroban native `require_auth()` for authorization.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cargo test --features testutils
```

The `testutils` feature enables Soroban test utilities. The `ed25519-dalek` dependency is pinned to v2.2.0 for compatibility.

## Deploy

Via Astinel backend API:

```bash
curl -X POST http://localhost:8080/v1/projects/{id}/contracts/deploy \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"contract_name": "astinel-access-control", "network": "testnet"}'
```

Or via Soroban CLI:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/astinel_access_control.wasm \
  --source $ACCOUNT --network testnet
```

## Contract Details

### astinel-access-control

RBAC with admin, operators, and pause/unpause.

**Storage**: ADMIN (`Address`), OPERATORS (`Vec<Address>`), PAUSED (`bool`)

| Function | Auth | Description |
|---|---|---|
| `init(admin)` | None | Initialize with admin, empty operators, unpaused |
| `admin()` | Read | Returns admin |
| `set_admin(caller, new_admin)` | Admin only | Transfer admin |
| `operators()` | Read | Returns operator list |
| `add_operator(caller, operator)` | Admin only | Add operator |
| `remove_operator(caller, operator)` | Admin only | Remove operator |
| `paused()` | Read | Returns paused flag |
| `set_paused(caller, paused)` | Admin only | Set pause flag |
| `require_not_paused()` | Read | Panics if paused |

### astinel-payable

ERC-20-like fungible token with mint, transfer, approve, transfer_from.

**Storage**: ADMIN, TOTLSUP (`i128`), balances (persistent per-address), allowances (persistent per `(owner, spender)` pair with `amount` + `expiration_ledger`)

| Function | Auth | Description |
|---|---|---|
| `init(admin)` | None | Initialize |
| `mint(caller, to, amount)` | Admin only | Mint (> 0) |
| `transfer(from, to, amount)` | From auth | Transfer |
| `approve(owner, spender, amount, expiration_ledger)` | Owner auth | Set allowance |
| `transfer_from(spender, from, to, amount)` | Spender auth | Spend allowance |

### astinel-ttl-management

Persistent key-value store with configurable per-entry TTL.

**Storage**: ADMIN, ENTRIES (`Vec<Symbol>`), data entries (persistent, `DataEntry { key, value: Bytes, ttl_ledgers, created_at }`)

| Function | Auth | Description |
|---|---|---|
| `write(caller, key, value, ttl_ledgers)` | Admin only | Store entry (ttl_ledgers > 0) |
| `read(key)` | Read | Read entry |
| `remove(caller, key)` | Admin only | Delete entry |
| `extend_ttl(caller, key, ttl_ledgers)` | Admin only | Update TTL |
| `entries()` | Read | List all keys |
| `remaining_ttl(key)` | Read | Compute remaining TTL |

### astinel-upgradeable

WASM-based contract upgrade with version enforcement.

**Storage**: ADMIN, IMPL (`BytesN<32>`), VERSION (`u32`), arbitrary instance storage

| Function | Auth | Description |
|---|---|---|
| `init(admin, wasm_hash, version)` | None | Initialize |
| `upgrade(caller, new_hash, new_version)` | Admin only | Upgrade (version must increase) |
| `set_admin(caller, new_admin)` | Admin only | Transfer admin |
| `write_bytes(caller, key, value)` | Admin only | Store arbitrary bytes |
| `read_bytes(key)` | Read | Read arbitrary bytes |

`update_current_contract_wasm` is guarded with `#[cfg(not(test))]`.

### astinel-stellar-asset

SAFE (Stellar Asset Frontend) standard token with mint, burn, transfer, approve, transfer_from.

**Storage**: ADMIN, METADATA (`AssetMetadata { name, symbol, decimals: 7, total_supply, issuer }`), TOTAL_SUP, balances, allowances

| Function | Auth | Description |
|---|---|---|
| `init(admin, name, symbol)` | None | Initialize |
| `mint(caller, to, amount)` | Admin only | Mint (> 0) |
| `burn(_caller, from, amount)` | From auth | Burn own tokens |
| `transfer(from, to, amount)` | From auth | Transfer |
| `approve(owner, spender, amount, expiration_ledger)` | Owner auth | Set allowance |
| `transfer_from(spender, from, to, amount)` | Spender auth | Spend allowance |

## License

MIT or Apache 2.0.
