# SoroWave Contracts

Soroban contract prototype for escrowing a shared token pool, recording contributor points, finalizing a cycle, and allowing contributors to claim pro-rata rewards. Experimental and unaudited. Use testnet only; do not deposit real funds.

## Contract actions

- Initialize an administrator and the token contract once.
- Create a wave pool by transferring tokens into escrow.
- Record points for completed contributions with administrator authorization; the app uses an application ID to make each award idempotent.
- Finalize the cycle after points are settled.
- Let each contributor claim their proportional share after finalization.
- Cancel and refund a pool only if no points have been awarded.
- Read pool totals, point totals, and claim status.

Payouts use integer division. Rounding remainder stays in the pool and is carried forward to later claimants; the final points claimant receives the remaining pool amount. Initialization, point awards, and finalization require admin authorization; claims require contributor authorization. The contract publishes lifecycle events and extends instance and persistent-entry TTLs when used. The SoroWave app calls the idempotent application-award method, reads pool and contributor state, and submits contributor claims after configuration. The contract is unaudited. Before production, add accounting and authorization tests, define dispute handling, and get an independent security review.

## Build

Install the Stellar CLI and Rust WebAssembly target, then run `stellar contract build` from this directory. See the [official Stellar contract guide](https://developers.stellar.org/docs/build/smart-contracts/overview).

## Testnet deployment outline

Create a disposable testnet identity, build and deploy with the Stellar CLI, then initialize the deployed contract with the administrator and a testnet token contract address. Record the deployed Soroban contract ID and token contract ID in `sorowave-api/.env` as `SOROWAVE_CONTRACT_ID` and `SOROWAVE_TOKEN_CONTRACT_ID`; set `SOROWAVE_WAVE_ID` to the numeric pool ID. Keep the identity secret out of the repositories and do not use a production key. No deployment is included because deployment and initialization require a funded testnet identity and explicit operator action. See the [Stellar CLI contract commands](https://developers.stellar.org/docs/tools/cli/stellar-cli).
