# Meteora Sniper Bot

The Meteora Sniper Bot is a Rust application that watches the Yellowstone gRPC Geyser feed for new Meteora liquidity pools and immediately submits buy transactions across multiple RPC providers. The bot automates the first purchase of WSOL pairs by building and dispatching transactions in parallel, giving you the best chance to win the race to the new pool.

## Features

- Subscribes to a Yellowstone gRPC endpoint for real-time Meteora pool creation events.
- Filters pools to only react to WSOL pairs, with an optional token mint allow-list.
- Builds swap transactions with configurable compute limits, priorities, and tips.
- Sends the transaction concurrently to Solana RPC, Jito, Bloxroute, and Nextblock endpoints (or any mix you configure).
- Uses a single Solana keypair that is stored locally in `config.yaml`.

## Prerequisites

- **Rust**: Install Rust toolchain `1.84.0` (matching [`rust-toolchain.toml`](./rust-toolchain.toml)). You can install it via [rustup](https://rustup.rs/).
- **Solana Keypair**: A funded Solana keypair encoded as a base58 string (use `solana-keygen grind --outfile key.json` followed by `solana-keygen pubkey key.json` or `solana-keygen recover` and `solana-keygen pubkey` to export the base58 string).
- **Access to RPC Providers**: URLs or API tokens for the Solana RPC providers you want to use (Jito, Bloxroute, Nextblock, etc.).
- **Yellowstone gRPC Access**: A valid gRPC endpoint URL and `X-Token` for the Yellowstone feed you intend to consume.

## Configuration

The bot reads `config.yaml` in the project root at startup. Begin by copying the example file:

```bash
cp config.example.yaml config.yaml
```

Edit `config.yaml` to match your environment:

| Field | Description |
| --- | --- |
| `rpc` | Map of named RPC destinations. Each entry should define a `url`, optional `auth` token, and `rpc_type` (`solanarpc`, `jito`, `bloxroute`, or `nextblock`). The bot will attempt to send the buy transaction to every configured RPC in parallel. |
| `geyser_url` | Yellowstone gRPC endpoint that serves account and transaction updates. |
| `geyser_x_token` | Authentication token to pass as the `x-token` header when connecting to the geyser service. Leave blank if not required. |
| `private_key` | Base58-encoded private key string for the Solana account that will sign the buy transaction. |
| `compute_unit_price` / `compute_unit_limit` | Compute budget instructions that will be inserted before the swap. Set to `0` to omit. |
| `tip` | Amount of SOL to tip when submitting through Jito, Bloxroute, or Nextblock. (Ignored for standard Solana RPCs.) |
| `buy_amount` | Amount of SOL (in whole SOL) to swap for the target token. |
| `min_amount_out` | Minimum amount of the target token expected from the swap (denominated in token base units). |
| `buy_token_mint` | Optional mint address to restrict the bot to a single target token. Leave blank to snipe every new WSOL pair. |

> **Note**: Any RPC that requires bearer authentication (e.g. Bloxroute or Nextblock) should have its `auth` field set to the API key or token the provider expects.

## Running the Bot

1. Ensure `config.yaml` is populated with working credentials and funding.
2. Set any desired logging level via `RUST_LOG`. For example, to enable informational logs:

   ```bash
   export RUST_LOG=info
   ```

3. Start the bot with Cargo:

   ```bash
   cargo run --release
   ```

   The application will:

   - Connect to the Yellowstone gRPC stream.
   - Listen for Meteora pool creation instructions that involve WSOL and match your optional mint filter.
   - Build the swap transaction using your keypair and configuration values.
   - Dispatch the transaction to every configured RPC sender concurrently.

4. Monitor the logs for confirmation messages such as `bench complete!` and RPC latencies.

## Deployment Tips

- Keep your `config.yaml` outside of version control and protect it carefully—the file contains your private key.
- Run the bot on a low-latency server close to your chosen RPC providers and the Yellowstone endpoint for best results.
- Regularly top up the signing account so that compute fees, tips, and swap amounts can be paid without interruption.
- Consider using a process supervisor (systemd, Docker, etc.) to keep the bot running and automatically restart it on crashes or network blips.

## Troubleshooting

- **Connection errors to Yellowstone**: Double-check `geyser_url` and `geyser_x_token`. Some providers require TLS; the sample configuration uses HTTPS.
- **Transaction failures**: Inspect the logs for RPC-specific errors. You may need to adjust `compute_unit_price`, `tip`, or `min_amount_out` to match network conditions.
- **Missing swap attempts**: Ensure `buy_token_mint` is blank unless you intend to target a single mint, and verify that the new pool pairs WSOL on the B side (as required by the current logic).

## License

This project does not currently specify a license. Verify the terms with the repository owner before using it in production.
