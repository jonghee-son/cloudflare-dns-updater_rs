# Cloudflare DNS updater_rs
Cloudflare DNS Record updater written in rust.
Manage your custom domain without static ip.

## Setup

1. Install Rust (edition 2024 compatible).
2. Update `.env` file in the project root with your credentials:

```
CLOUDFLARE_EMAIL=you@example.com
CLOUDFLARE_API_KEY=your_api_key_or_token
CLOUDFLARE_DOMAIN=example.com
```

3. Build or run:

```
cargo run
# or
cargo build -r
```

## What it does

- Loads credentials from environment variables (or `.env`).
- Looks up the zone and A record for `CLOUDFLARE_DOMAIN` via Cloudflare API.
- Fetches your current public IPv4 from https://checkip.amazonaws.com.
- Updates the A record when the IP changes (every 10 minutes by default).

## Notes

- Uses `curl` bindings; OpenSSL is vendored by default. On Windows, ensure `perl` and `nasm` are available when building.
- Only IPv4 is supported currently.
