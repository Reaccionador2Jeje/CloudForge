# CloudForge Web Console — milestone 1.1

## Overview

The console is a responsive HTML/CSS/JavaScript application embedded into the
Rust API Gateway for local development and published as static assets on
Cloudflare Workers for the public website.

Locally, the API Gateway serves the console at:

- `GET /`
- `GET /console`

On Cloudflare Workers, the static console is served from Workers Static Assets.
Requests to approved `/health/*` and `/v1/*` routes are handled by the Worker
proxy and forwarded to a separate HTTPS API origin protected by Cloudflare
Access. The browser uses same-origin API paths; it does not contain upstream
hostnames or Access credentials.

## Available screens

- **Overview:** API liveness/readiness, API version, database state, and
  instance count.
- **Instances:** list metadata records and create pending metadata records
  after login. This does not provision a VM.
- **Nodes:** inspect registered node metadata and review the current enrollment
  readiness. Remote mTLS enrollment and heartbeats are not implemented.
- **Components:** search and filter the API component capability registry.
- **Diagnostics:** probe health, system information, instance, and node API
  routes.

## Authentication

The local API includes one-time first-admin bootstrap, Argon2id password
hashing, login/logout, 12-hour bearer sessions, and initial RBAC middleware.
The browser keeps the bearer token in memory only; closing or reloading the
page clears it.

The first administrator can be created only while the users table is empty
and the API host has `CLOUDFORGE_BOOTSTRAP_TOKEN` configured. Never put this
token in frontend code or the public repository.

## Run locally

Start PostgreSQL if you want to test database-backed records:

```bash
docker compose up -d postgres
```

Configure `.env`, then run:

```bash
cargo run -p cloudforge-api-gateway
```

Open `http://127.0.0.1:8080/` or
`http://127.0.0.1:8080/console`.

## Public Cloudflare deployment

The Worker proxy requires:

- `CLOUDFORGE_API_ORIGIN`: an HTTPS hostname routed through Cloudflare Tunnel
  and protected by Cloudflare Access.
- `CF_ACCESS_CLIENT_ID`: a Cloudflare Access Service Token client ID.
- `CF_ACCESS_CLIENT_SECRET`: the corresponding secret.

The two Access values must be Worker secrets. The proxy allows only the
specific paths and HTTP methods defined in `worker/index.js`. It fails closed
with HTTP 503 when the backend or Access credentials are missing. The console
does not show simulated sample rows when the API is unavailable.

See [Cloudflare Workers deployment](deployment/cloudflare-workers.md) for the
complete topology and configuration steps.

## Security and implementation limits

The current authentication and RBAC are a development foundation, not a
production security certification. Rate limiting, account recovery, full user
and role administration, complete audit-query tooling, and a security review
remain pending. Keep the Rust API bound to loopback behind Cloudflare Tunnel
and Access; do not expose port 8080 directly to the public Internet.

The instance registry stores metadata only. It does not provision or start
virtual machines. The Nodes screen does not enroll agents, accept certificates,
or create node credentials. QEMU/KVM, containerd, storage providers, and
network providers remain planned components until implemented and tested.

## Component catalog

The Components screen calls `GET /v1/system/components`. A `planned` state
means that the subsystem is a roadmap item, not an active backend service.
