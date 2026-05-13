<p align="center">
  <h1 align="center">isdown</h1>
  <p align="center">Check if a service is down. From your terminal.</p>
</p>

<p align="center">
  <a href="https://github.com/dariush624/isdown/actions/workflows/rust.yml"><img src="https://github.com/dariush624/isdown/actions/workflows/rust.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/dariush624/isdown/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

---

A fast CLI tool that checks the live status of third-party services by querying their public status APIs. No API keys needed.

```
$ isdown check github slack

GitHub:  Up
Slack:   Degraded
  · Slow API (investigating)
```

## Installation

### From source (requires [Rust toolchain](https://rustup.rs/))

```sh
git clone https://github.com/dariush624/isdown.git
cd isdown
cargo install --path .
```


## Usage

### Check one or more services

```sh
isdown check github
isdown check github slack aws cloudflare
```

### Check any URL

```sh
isdown check https://example.com
isdown check github https://my-api.example.com
```

### JSON output

```sh
isdown check github slack --json
```

```json
[
  {
    "Success": {
      "provider": "GitHub",
      "status": "Up",
      "causes": []
    }
  },
  {
    "Success": {
      "provider": "Slack",
      "status": "Up",
      "causes": []
    }
  }
]
```

### Custom timeout

```sh
isdown -t 5 check aws
```

## Options

| Flag              | Default | Description                |
| ----------------- | ------- | -------------------------- |
| `-t`, `--timeout` | `10`    | Request timeout in seconds |
| `-j`, `--json`    | off     | Output results as JSON     |

## How It Works

`isdown` queries public status APIs for each service:

- **Statuspage-based services** (GitHub, Cloudflare, Discord, etc.) use the [Atlassian Statuspage](https://www.atlassianstatuspage.com/) `/api/v2/summary.json` endpoint
- **AWS** parses the [AWS Health Dashboard](https://health.aws.amazon.com/) events feed, reporting per-service and per-region incidents
- **URL targets** perform a simple HTTP GET and report Up (200) or Down (non-200)

## Contributing

Contributions are welcome! To add a new Statuspage-based service, it's a one-liner:

```rust
// in src/check/statuspageio.rs
statuspage_provider!(MyServiceCheck, "MyService", "status.myservice.com");
```

Then register it in `src/registry.rs` and `src/check.rs`.

```sh
# Run tests
cargo test

# Run lints
cargo clippy
cargo fmt --check
```

## License

[MIT](LICENSE)
