# isdown

[![Rust](https://github.com/dariush624/isdown/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/dariush624/isdown/actions/workflows/rust.yml)

> Work in progress

A CLI tool that checks the live status of third-party services by querying their public status APIs.

```
isdown check github
isdown check github slack aws
isdown check github --json
```

## Supported services

| Name | Provider |
|------|----------|
| `github` | GitHub |
| `slack` | Slack |
| `aws` | Amazon Web Services |
| `atlassian` | Atlassian |
| `circleci` | CircleCI |
| `cloudflare` | Cloudflare |
| `datadog` | Datadog |
| `discord` | Discord |
| `linear` | Linear |
| `netlify` | Netlify |
| `npm` | npm |
| `openai` | OpenAI |
| `vercel` | Vercel |

## Output

```
GitHub: Up
Slack: Degraded
  · Slow API (investigating)
AWS: Down
  · S3 errors (us-east-1)
```

Pass `--json` for machine-readable output.

## Options

| Flag              | Default | Description |
|-------------------|---------|-------------|
| `-t`, `--timeout` | `10` | Request timeout in seconds |
| `--json`, `-j`     | off | Output results as JSON |

