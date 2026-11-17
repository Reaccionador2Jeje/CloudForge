# CloudForge

**CloudForge** is an open-source cloud infrastructure platform developed by **Atdev Dev**.

Its long-term goal is to let operators build, deploy, and manage virtual machines, containers, storage, networking, and cloud services on infrastructure they own and operate.

CloudForge is designed as a modular infrastructure platform, not merely a hosting control panel. It separates the control plane, which coordinates resources and desired state, from the data plane, where workloads run on physical nodes.

## Project goals

- Manage physical compute nodes and their available capacity.
- Provision and operate virtual machines using QEMU/KVM.
- Support container workloads.
- Provide persistent storage, volumes, snapshots, and backups.
- Manage virtual networks, IP addressing, and firewall policies.
- Expose documented APIs and cross-platform management clients.
- Build security, auditability, and least-privilege access into the architecture.

## Technology direction

| Area | Technology |
| --- | --- |
| Core services | Rust |
| HTTP API | Axum |
| Async runtime | Tokio |
| Virtualization | QEMU/KVM |
| Containers | containerd |
| Database | PostgreSQL |
| Storage | ZFS initially; evaluate Ceph for distributed deployments |
| Web client | TypeScript |
| Mobile client | Flutter |

These are architectural choices and targets, not a claim that every component is already implemented.

## Repository layout

```text
core/
  api-gateway/       Initial HTTP API service
docs/
  architecture/      System architecture
  decisions/         Architecture Decision Records
.github/
  workflows/         Continuous integration
```

## Current status

**Milestone 0.2 — Persistent Cloud Core**

- [x] Rust workspace
- [x] Axum API Gateway skeleton
- [x] Liveness and readiness endpoints
- [x] Environment-based listen configuration
- [x] Structured logging
- [x] Graceful shutdown
- [x] Initial architecture documentation
- [x] Rust CI workflow
- [x] Optional PostgreSQL integration and migrations
- [x] Initial persistent instance registry API
- [x] Structured API error responses
- [x] Local PostgreSQL Compose environment
- [ ] Identity and authorization
- [ ] Node Agent
- [ ] QEMU/KVM lifecycle management

The current API Gateway is a development foundation. It does not yet provision virtual machines or manage production infrastructure.

## Development

Requirements:

- Rust stable toolchain
- Cargo
- Docker Compose (for local PostgreSQL)

Start PostgreSQL:

```bash
docker compose up -d postgres
```

Copy `.env.example` to `.env` and adjust it if necessary. The supplied database password is for local development only.

Run the API Gateway:

```bash
cargo run -p cloudforge-api-gateway
```

By default, the API listens on `127.0.0.1:8080`. When `DATABASE_URL` is configured, the gateway connects to PostgreSQL and applies embedded SQL migrations at startup. Without it, health and system-information endpoints remain available, while database-backed endpoints return `503 Service Unavailable`.

Configuration:

| Variable | Default | Description |
| --- | --- | --- |
| `CLOUDFORGE_HOST` | `127.0.0.1` | Bind address |
| `CLOUDFORGE_PORT` | `8080` | TCP port |
| `CLOUDFORGE_DATABASE_MAX_CONNECTIONS` | `5` | PostgreSQL connection pool limit |
| `DATABASE_URL` | unset | Optional PostgreSQL connection string |
| `RUST_LOG` | `info` | Logging filter |

Health and information endpoints:

- `GET /health/live`
- `GET /health/ready`
- `GET /v1/system/info`

Initial instance registry endpoints (PostgreSQL required):

- `GET /v1/instances` — list registered instance records.
- `POST /v1/instances` — create a pending registry record using JSON: `{"name":"my-server"}`.

The instance registry stores metadata only. It does **not** create or start virtual machines. Actual provisioning will be implemented after identity, authorization, and node-agent boundaries are in place.

Run quality checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Security

CloudForge is infrastructure software. Remote host operations must use authenticated service identities, explicit authorization, narrow operation interfaces, audit records, and secure communication. Do not expose node agents directly to untrusted networks.

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE).

## Project

Developed by **Atdev Dev**.


## Milestone 0.3 — Node Agent Foundation

The standalone Rust crate `node/node-agent` collects a read-only host inventory and prints a versioned JSON snapshot:

```bash
cargo run -p cloudforge-node-agent
```

It reports operating system, kernel, architecture, logical CPU count, memory, and mounted filesystem capacity. It runs without elevated privileges, opens no network listener, executes no remote commands, and sends no data to the API Gateway. Inventory values are observations, not scheduler reservations.

The authenticated registration and heartbeat protocol is intentionally deferred until node identity, credential rotation/revocation, TLS validation, and authorization are implemented. See [Node Agent architecture](docs/architecture/node-agent.md).


## Milestone 0.4 — Node Identity Preflight

The Node Agent now includes an identity configuration preflight:

```bash
cargo run -p cloudforge-node-agent -- --check-identity
```

Configure the following environment variables before running it:

| Variable | Purpose |
| --- | --- |
| `CLOUDFORGE_NODE_ID` | Stable node UUID |
| `CLOUDFORGE_CONTROL_PLANE_URL` | Control Plane HTTPS URL |
| `CLOUDFORGE_NODE_CLIENT_CERT` | Client certificate PEM path |
| `CLOUDFORGE_NODE_CLIENT_KEY` | Private key PEM path |
| `CLOUDFORGE_NODE_CA_CERT` | Trusted CA certificate PEM path |

The preflight rejects non-HTTPS endpoints, checks required files and PEM markers, and on Unix requires owner-only private-key permissions. Its output contains only a safe configuration summary.

**This is not remote authentication or enrollment yet.** It does not establish TLS, validate certificate chains, contact the API Gateway, or register a node. Actual mTLS transport, server-side certificate binding, authorization, revocation, and heartbeat persistence remain future work. See [Node identity and secure enrollment](docs/architecture/node-identity.md).
