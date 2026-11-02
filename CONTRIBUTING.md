# Contributing to SoroWave Contracts

SoroWave is an independent community preview. Listing an issue here does not enroll it in Drips Wave or reserve a reward.

## Workflow

1. Read the issue and acceptance criteria. Do not begin work until a maintainer assigns it on GitHub.
2. Keep contract changes focused and include the security and accounting implications in the pull request.
3. Reference the issue with `Closes #<number>` and explain the state transitions affected.
4. Never include a secret key, seed phrase, or funded wallet in a commit, issue, or pull request.

## Local checks

Install Rust and the Stellar CLI. Run `cargo check --locked` for a Rust compile check and `stellar contract build` to produce the contract WASM. This unaudited prototype is testnet-only and must not hold real funds.
