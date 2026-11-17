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

**Milestone 0.1 — Foundation**

- [x] Rust workspace
- [x] Axum API Gateway skeleton
- [x] Liveness and readiness endpoints
- [x] Environment-based listen configuration
- [x] Structured logging
- [x] Graceful shutdown
- [x] Initial architecture documentation
- [x] Rust CI workflow
- [ ] PostgreSQL integration
- [ ] Identity and authorization
- [ ] Node Agent
- [ ] QEMU/KVM lifecycle management

The current API Gateway is a development foundation. It does not yet provision virtual machines or manage production infrastructure.

## Development

Start the local PostgreSQL dependency (optional for health endpoints, required for instance persistence):

\`\`\`bash
docker compose up -d postgres
\`\`\`

Copy \`.env.example\` to \`.env\`, then start the API:

\`\`\`bash
cargo run -p cloudforge-api-gateway
\`\`\`

When \`DATABASE_URL\` is configured, the gateway applies SQL migrations at startup. The local Compose password is for development only; never expose this database or reuse that password in production.

### Initial instance registry API

- \`GET /v1/instances\` — list registered instance records.
- \`POST /v1/instances\` — create a pending registry record using JSON: \`{"name":"my-server"}\`.

These endpoints manage database records only. They do **not** create or start virtual machines yet. Real provisioning will be introduced after the identity, authorization, and node-agent boundaries are designed.


Requirements:

- Rust stable toolchain
- Cargo

Run the API Gateway:

```bash
cargo run -p cloudforge-api-gateway
```

By default, it listens on `127.0.0.1:8080`.

Configuration:

| Variable | Default | Description |
| --- | --- | --- |
| `CLOUDFORGE_HOST` | `127.0.0.1` | Bind address |
| `CLOUDFORGE_PORT` | `8080` | TCP port |
| `RUST_LOG` | `info` | Logging filter |

Health endpoints:

- `GET /health/live`
- `GET /health/ready`

The readiness endpoint currently checks only that the HTTP service is running. Dependency checks will be added with database and internal-service integration.

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
