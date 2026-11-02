# SoroWave Contracts

Soroban contract prototype for holding a Stellar token bounty until a maintainer approves a contribution. This repository is experimental and unaudited. Use testnet only; do not deposit real funds.

## Contract actions

- Initialize an administrator and the token contract once.
- Create a bounty by transferring tokens into escrow.
- Release a funded bounty to a contributor after the maintainer approves the work.
- Cancel a bounty and return its funds to its creator.

The contract is intentionally a small starting point. Before production, add comprehensive authorization and edge-case tests, review token semantics, define dispute handling, and get an independent security review.

## Build

Install the Stellar CLI and Rust WebAssembly target, then run `stellar contract build` from this directory. See the [official Stellar contract guide](https://developers.stellar.org/docs/build/smart-contracts/overview).
