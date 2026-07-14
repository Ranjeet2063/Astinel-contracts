# Contributing to Astinel Contracts

## Development Setup

```bash
git clone https://github.com/Astinel-Org/Astinel-contracts.git
cd Astinel-contracts
rustup target add wasm32-unknown-unknown
cargo build
```

## CI Expectations

Every pull request must pass:

```bash
cargo build --target wasm32-unknown-unknown --release  # WASM build
cargo test --features testutils                         # All tests
cargo clippy -- -D warnings                             # Linting
cargo fmt --check                                       # Formatting
```

## Writing Contracts

- All contracts must be `#![no_std]` compatible
- Use `Symbol` with `symbol_short!()` for storage keys (max 6 characters)
- Every mutable function must call `caller.require_auth()`
- Use instance storage for configuration data, persistent storage for user data
- Emit events where appropriate for off-chain indexing

## Writing Tests

- Use `#[cfg(test)]` module with `soroban_sdk::testutils`
- Register contracts with `env.register()`
- Use `env.mock_all_auths()` for authorization in tests
- Use `env.as_contract()` to execute within the contract context
- Generate addresses with `Address::generate(&env)`
- Test both success and failure paths

## Commit Conventions

```
feat: add burn function to stellar-asset contract
fix: prevent double operator addition in access-control
test: add edge case tests for allowance expiration
docs: document TTL mechanics in README
```
