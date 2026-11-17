# CloudForge Console — Cloudflare Workers deployment

The public console is deployed as a **static-assets-only Cloudflare Worker** named `cloudforge`. Wrangler uploads the contents of `web/console/`; no Rust service, database, credentials, or node-agent interface is deployed with this site.

## Public demo behavior

- A persistent notice states that the API Gateway is not connected.
- Example instance rows are marked as simulated.
- Instance creation is disabled.
- Diagnostics do not send requests to API endpoints.
- No infrastructure health or capacity is represented as live data.

The Rust API Gateway remains a separate service. Do not point this public static console at an unauthenticated API. Before connecting a remote API, implement user authentication, authorization, CORS policy, rate limiting, and secure deployment configuration.

## Requirements

- Node.js and npm (or a compatible Node.js package runner).
- A Cloudflare account authorized for Workers.
- Wrangler 4.

## Deploy from the repository root

```bash
npx wrangler@4 deploy
```

For a local preview:

```bash
npx wrangler@4 dev
```

Wrangler reads `wrangler.jsonc` and publishes the static files from `web/console/` to the `cloudforge` Worker. With `workers_dev` enabled, Cloudflare assigns a URL under `workers.dev` on the account configured subdomain.

## Verify

After deployment, open the URL printed by Wrangler and confirm:

1. The demo notice is visible.
2. The status says `Modo demo · API no conectada`.
3. The instances are clearly labelled as simulated.
4. Instance creation is disabled.
5. Diagnostics explicitly say no API request was sent.

A successful upload confirms publication of the static console only. It does not deploy or start the Rust API Gateway, PostgreSQL, QEMU/KVM, or Node Agents.
