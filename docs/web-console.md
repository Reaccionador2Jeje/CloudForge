# CloudForge Web Console — milestone 0.5

## Overview

The development console is a static, responsive HTML/CSS/JavaScript application embedded into the API Gateway binary at compile time. The gateway serves it at:

- `GET /`
- `GET /console`

Because the console and API share the same origin, the browser can call the existing API routes without a separate frontend server or permissive CORS configuration.

## Available screens

- **Overview:** liveness, readiness, API version, database state, and instance count.
- **Instances:** list metadata records and create a pending metadata record.
- **Nodes:** explains the current Node Agent preflight and clearly shows that remote enrollment is not yet available.
- **Diagnostics:** probes the health, system information, and instance API routes.

## Run locally

Start PostgreSQL if you want to test database-backed instance records:

```bash
docker compose up -d postgres
```

Configure `.env`, then run:

```bash
cargo run -p cloudforge-api-gateway
```

Open `http://127.0.0.1:8080/` or `http://127.0.0.1:8080/console`.

## Important security boundary

This is a development console, not an authenticated production admin panel. The existing instance API is not yet protected by user authentication and authorization. Keep the API bound to loopback (`CLOUDFORGE_HOST=127.0.0.1`) and do not expose it to the public Internet or an untrusted network.

The console only creates and lists instance metadata; it does not provision virtual machines. The Nodes screen is informational. It does not enroll agents, accept certificates, or create node credentials.

Before remote or multi-user deployment, CloudForge must implement user identity, session security, role-based authorization, CSRF protections for browser mutations where applicable, TLS termination, audit logging, and authenticated node enrollment.
