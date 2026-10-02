# btc-cli

A command-line toolkit for common Bitcoin tasks: keys, addresses,
transactions, and chain queries.

Built for developers who want quick, scriptable access to everyday Bitcoin
operations from the terminal. Offline commands (key generation, address
derivation, transaction decoding) work without a node. Online commands
(broadcasting, block lookup, fee estimation) connect to a Bitcoin Core node
over JSON-RPC.

Human-readable output by default. `--json` for machine-readable output.

## Status

MVP. Supports **regtest**, **signet**, and **testnet4**. Mainnet is
deliberately not supported yet.

## Commands

### Offline

| Command | Description |
|---|---|
| `key generate` | Generate a fresh secp256k1 keypair |
| `mnemonic new` | Generate a BIP39 mnemonic |
| `derive` | Derive child keys/addresses from an xprv or xpub |
| `address from-pubkey` | Derive an address from a public key |
| `address validate` | Validate an address and report its type and network |
| `tx decode` | Decode a raw transaction hex |
| `tx create` | Build an unsigned transaction from inputs and outputs |
| `tx sign` | Sign an unsigned transaction with one private key |

### Online (require a Bitcoin Core node)

| Command | Description |
|---|---|
| `tx broadcast` | Broadcast a signed transaction |
| `block info` | Fetch block details by height or hash |
| `fee estimate` | Get a fee rate estimate from the node |

## Global flags

| Flag | Default | Description |
|---|---|---|
| `--network <net>` | `regtest` | One of `regtest`, `signet`, `testnet4` |
| `--json` | off | Machine-readable JSON output |
| `--show-secret` | off | Include private keys / WIFs in output |

Global flags can appear anywhere on the command line.

## Installation

Requires a Rust toolchain (1.75+).

```bash
git clone <repo-url> btc-cli
cd btc-cli
cargo build --release