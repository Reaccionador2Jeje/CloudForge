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

**Milestone 1.1 — Protected Cloudflare API proxy foundation**

- [x] Rust workspace and Axum API Gateway
- [x] Optional PostgreSQL integration and instance/node metadata registries
- [x] Read-only local Node Agent inventory and identity preflight
- [x] Local user bootstrap, Argon2id login, bearer sessions, RBAC, and audit events
- [x] Cloudflare Worker entry point with allowlisted API proxy routes
- [x] HTTPS-only upstream validation and Cloudflare Access Service Token enforcement
- [x] Public console no longer fabricates sample infrastructure data
- [x] Add a Docker image and Compose deployment for the API Gateway and PostgreSQL
- [ ] Deploy and protect a real API Gateway origin behind Cloudflare Tunnel and Access
- [ ] Production authentication hardening and security review
- [ ] QEMU/KVM, containerd, storage, networking, and node mTLS

The Cloudflare Worker and console code are prepared for a protected API connection. A Dockerfile and Compose service now package the Rust API Gateway with PostgreSQL for a Linux host. The host still needs to be provisioned and connected through Cloudflare Tunnel and Access. See [API host deployment](docs/deployment/api-host.md). Until that is configured, the public console reports the API as unavailable instead of displaying simulated data.

## Development

Requirements:

- Rust stable toolchain
- Cargo
- Docker Compose (for local PostgreSQL)

Start the API Gateway and PostgreSQL together on a Linux host:

```bash
docker compose up -d --build
```

For the complete protected-host deployment, including Cloudflare Tunnel and Access, see [API host deployment](docs/deployment/api-host.md).

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
- `GET /v1/system/components` — read-only capability and implementation-status registry.

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

## Component integration status

The API exposes `GET /v1/system/components` as a read-only, versioned-by-API capability inventory. It distinguishes available services from optional, local-only, preflight-only, and planned components. This endpoint is descriptive: it does not imply that planned components are running or connected.

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

The preflight rejects non-HTTPS endpoints, checks required files and PEM markers, and on Unix requires owner-only private-key permissions. Its output reports that file preflight passed while explicitly stating that authentication has not been established and no TLS session exists.

**This is not remote authentication or enrollment yet.** It does not establish TLS, validate certificate chains, contact the API Gateway, or register a node. Actual mTLS transport, server-side certificate binding, authorization, revocation, and heartbeat persistence remain future work. See [Node identity and secure enrollment](docs/architecture/node-identity.md).


## Milestone 0.5 — Web Console

CloudForge now includes a responsive development console served directly by the API Gateway:

- Open `http://127.0.0.1:8080/` or `/console` after starting the gateway.
- Overview: API liveness/readiness, version, database state, and instance count.
- Instances: create and list pending metadata records.
- Nodes: Node Agent setup information and enrollment status.
- Diagnostics: test the current API endpoints from the browser.

The console is a static HTML/CSS/JavaScript application embedded into the gateway binary for local development. It can also be published on Cloudflare Workers, where a same-origin Worker proxy forwards an allowlist of API routes to a separate HTTPS API origin protected by Cloudflare Access. The public console no longer displays simulated sample rows.

**Security:** local user authentication, bearer sessions, and RBAC are implemented as an initial development foundation. Production rate limiting, account recovery, security review, and operational hardening remain pending. Keep the Rust API bound to `127.0.0.1` behind Cloudflare Tunnel and Access; do not expose port 8080 directly to the Internet. See [Web Console documentation](docs/web-console.md) and [Cloudflare Workers deployment](docs/deployment/cloudflare-workers.md).

## Milestone 0.6 — Node Registry Foundation

The PostgreSQL schema now includes a node registry foundation through
`0002_create_nodes.sql`. It stores stable node IDs, names, lifecycle states
(`pending`, `active`, `quarantined`, `revoked`), optional agent version,
verified-certificate fingerprint, inventory snapshot, and heartbeat timestamp.

This is database groundwork only. No node enrollment or heartbeat endpoint is
enabled, and no node is trusted or marked active from an unauthenticated report.
The Node Agent remains local and read-only. See
[Node Registry data model](docs/architecture/node-registry.md).


## Milestone 0.7 — Node Agent Protocol Design

The proposed communication contract is now documented in
[Node Agent communication protocol](docs/architecture/node-agent-protocol.md).
It specifies the future mTLS trust model, node identity binding, versioned
message envelope, replay/idempotency expectations, validation limits, and
rollout gates.

This milestone is documentation and truthful preflight reporting only. It does
not add a network client, TLS session, enrollment endpoint, heartbeat endpoint,
or remote node operations.


## Milestone 0.8 — Console Design System and Component Catalog

The console now uses a dedicated responsive stylesheet at
`web/console/styles.css`, with expanded navigation, mobile layouts, cards,
tables, status pills, component cards, search, and state filtering.

The new **Components** screen reads `GET /v1/system/components` from the API
Gateway. The public console no longer substitutes a simulated component
catalog when the API is unavailable. The API catalog now tracks the control plane, node registry, identity,
mTLS, compute, containers, scheduling, images, storage, networking,
observability, audit, and client integrations.

The catalog is a truthful implementation-status inventory. Components marked
`planned` are not running services and do not provision infrastructure.


## Milestone 0.9 — Shared Security Schema and Full Component Map

The capability catalog now tracks 41 platform subsystems across the control
plane, data plane, security, observability, clients, and extensibility. The
public demo catalog mirrors the API catalog and labels future providers as
planned rather than active.

Migration 0003_identity_rbac_audit.sql adds foundational PostgreSQL tables
for users, roles, permissions, role assignments, hashed session-token storage,
and append-only audit events. It seeds role definitions only; it does not
create users, assign privileges, enable login, or activate authorization
middleware.

See the full component map and implementation order in
docs/architecture/component-map.md.
QEMU/KVM, containerd, and storage providers remain intentionally deferred until
identity, authorization, node trust, provider-neutral APIs, and scheduling
foundations have been built and tested.


## Milestone 1.0 — Local Identity, Sessions, RBAC, and Audit

The API Gateway now provides:

- One-time first-administrator bootstrap, guarded by the
  `CLOUDFORGE_BOOTSTRAP_TOKEN` environment variable (minimum 32 characters).
- Argon2id password hashing and password verification.
- Login and logout endpoints with random 256-bit bearer tokens.
- Only SHA-256 token hashes are stored in PostgreSQL; sessions expire after
  12 hours and can be revoked.
- RBAC middleware protects instance listing/creation and node listing.
- Authentication successes, failures, bootstrap attempts, and permission
  denials are written to the append-only audit table.

### Local first-time setup

Generate a unique bootstrap token and keep it private:

```bash
openssl rand -hex 32
```

Set the generated value as `CLOUDFORGE_BOOTSTRAP_TOKEN` in the local
environment or an untracked `.env` file, start PostgreSQL, and start the API
Gateway. Then make the one-time request:

```bash
curl -X POST http://127.0.0.1:8080/v1/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"bootstrap_token":"YOUR_BOOTSTRAP_TOKEN","email":"admin@example.com","password":"YOUR_LONG_PASSWORD","display_name":"CloudForge Admin"}'
```

The response contains a bearer token. Send it as
`Authorization: Bearer YOUR_ACCESS_TOKEN` when calling protected endpoints.
Use `POST /v1/auth/login`, `GET /v1/auth/me`, and `POST /v1/auth/logout`
for session management. Bootstrap can succeed only once, while the users table
is empty.

**Deployment boundary:** this is an initial local-development security
milestone, not a public-production authentication system. Login rate limiting,
TLS termination, CSRF protections for browser-based credentials, account
recovery, key rotation, and a full security review are still required before
exposing the API to untrusted networks. Keep `CLOUDFORGE_HOST=127.0.0.1`.
Never commit the bootstrap token, passwords, bearer tokens, or a populated
`.env` file.


## Milestone 1.1 — Protected Cloudflare API Proxy

The public Cloudflare Worker now serves the static console and routes only
approved `/health/*` and `/v1/*` endpoints through a server-side proxy.
The proxy validates an HTTPS upstream origin, requires a Cloudflare Access
Service Token, restricts each endpoint to its intended HTTP methods, and
returns an explicit unavailable response when the backend or credentials are
not configured.

The browser no longer contains a public demo switch or simulated instance
rows. It queries the real API contract and shows an unavailable state until
the protected backend is connected.

The Worker still does not host the Rust API or database. Deployment requires
a Linux API host, private PostgreSQL, Cloudflare Tunnel, an Access application
with a Service Auth policy, and Worker configuration values/secrets. See
[Cloudflare Workers deployment](docs/deployment/cloudflare-workers.md).
