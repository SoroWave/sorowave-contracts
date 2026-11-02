# SoroWave Contracts

Soroban contract prototype for holding a Stellar token bounty until a maintainer approves a contribution. This repository is experimental and unaudited. Use testnet only; do not deposit real funds.

## Contract actions

- Initialize an administrator and the token contract once.
- Read a bounty by its numeric identifier.
- Create a bounty by transferring tokens into escrow.
- Release a funded bounty to a contributor after the maintainer approves the work.
- Cancel a bounty and return its funds to its creator.

Initialization requires the administrator's authorization. The contract publishes lifecycle events. The contract is intentionally a small starting point and is unaudited. Before production, add comprehensive authorization and edge-case tests, review token semantics, define dispute handling, and get an independent security review.

## Build

Install the Stellar CLI and Rust WebAssembly target, then run `stellar contract build` from this directory. See the [official Stellar contract guide](https://developers.stellar.org/docs/build/smart-contracts/overview).

## Testnet deployment outline

Create a disposable testnet identity, build and deploy with the Stellar CLI, then initialize the deployed contract with the administrator and a testnet token contract address. Record the deployed Soroban contract ID and token contract ID in `sorowave-api/.env` as `SOROWAVE_CONTRACT_ID` and `SOROWAVE_TOKEN_CONTRACT_ID`. Keep the identity secret out of the repositories and do not use a production key. No deployment is included in this starter because deployment and initialization require a funded testnet identity and explicit operator action. See the [Stellar CLI contract commands](https://developers.stellar.org/docs/tools/cli/stellar-cli).
