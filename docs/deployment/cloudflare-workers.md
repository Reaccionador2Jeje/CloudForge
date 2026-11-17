# CloudForge Console — Cloudflare Workers deployment

CloudForge uses a Cloudflare Worker for the public web console and as a **same-origin, allowlisted API proxy**. The Rust API Gateway remains a separate service. The Worker does not run Rust, PostgreSQL, QEMU/KVM, containerd, or storage providers.

## Request flow

1. The browser loads the static console from Workers Static Assets.
2. Browser requests to `/health/*` and `/v1/*` are routed through `worker/index.js`.
3. The Worker accepts only explicitly listed API paths and HTTP methods.
4. It forwards requests to `CLOUDFORGE_API_ORIGIN` over HTTPS and adds a Cloudflare Access Service Token.
5. Cloudflare Access protects the API hostname before requests reach the Rust API Gateway.

The Worker fails closed with HTTP 503 if the API origin or Access credentials are missing. It does not silently switch to simulated data. Browser requests are same-origin, so the console does not need permissive CORS rules.

## Required backend topology

Use a Linux host or server that can run the Rust API Gateway and PostgreSQL. Keep the API bound to loopback:

- API Gateway: `127.0.0.1:8080`
- Cloudflare Tunnel origin service: `http://127.0.0.1:8080`
- Public API hostname: a dedicated HTTPS hostname, protected by Cloudflare Access.

Do not expose port 8080 directly to the Internet. Cloudflare Tunnel should provide the network path to the host, and Cloudflare Access should require a Service Auth policy for the API hostname. Create a Cloudflare Access Service Token for that policy.

The API host must also have PostgreSQL configured before database-backed routes are useful. Keep the database private and use a strong, unique password.

## Configure the Worker

The repository configures:

- Worker name: `cloudforge`
- Static assets: `web/console/`
- Worker entry point: `worker/index.js`
- API routing: `/health/*` and `/v1/*`

In the Cloudflare dashboard, open the `cloudforge` Worker settings and configure:

**Variable (non-secret):**

- `CLOUDFORGE_API_ORIGIN`: the HTTPS origin of the API hostname protected by Cloudflare Access, for example `https://api.example.com`. Enter only the origin; do not include a path, query, or credentials.

**Secrets:**

- `CF_ACCESS_CLIENT_ID`: the Service Token client ID.
- `CF_ACCESS_CLIENT_SECRET`: the Service Token client secret.

Never put these values in Git, the browser, README files, or chat. Do not use a Cloudflare API token as an Access Service Token; they are different credentials.

The Worker only forwards a fixed set of endpoints and methods. It does not accept arbitrary upstream URLs, preventing the browser from turning the Worker into a general-purpose proxy.

## Deploy from GitHub Actions

This is the recommended deployment path for Android/Termux because Wrangler's local runtime is not supported on Android ARM64.

1. In `Reaccionador2Jeje/CloudForge`, open **Settings → Secrets and variables → Actions**.
2. Confirm the repository secrets `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` exist. Keep them private.
3. Open **Actions → Deploy CloudForge Console → Run workflow**.
4. Select `main` and run the workflow.
5. Wait for the deploy job to complete, then open the Worker URL.

The deployment publishes the console and proxy code. It does **not** create a Tunnel, an Access application, a Service Token, a Rust API host, or a PostgreSQL server.

## Configure the Rust API host

On the Linux API host:

1. Configure `DATABASE_URL` for the private PostgreSQL service.
2. Set `CLOUDFORGE_HOST=127.0.0.1` and the desired API port.
3. Generate a long random `CLOUDFORGE_BOOTSTRAP_TOKEN` and keep it in the API host's environment, never in the frontend or repository.
4. Start the API Gateway and verify `http://127.0.0.1:8080/health/ready` locally.
5. Configure Cloudflare Tunnel to route the protected API hostname to `http://127.0.0.1:8080`.
6. Configure a Cloudflare Access application and a Service Auth policy for that hostname. The Worker Service Token must satisfy that policy.

The public console's first-admin form sends the bootstrap token directly to the API through the protected Worker proxy. Bootstrap succeeds only once, while the users table is empty.

## Verify

Before setting the Worker variables and secrets, the public console should show that the API is unavailable. This is intentional and confirms that it is not inventing data.

After the origin and Access Service Token are configured:

1. The console should report the API Gateway online.
2. The component catalog should load from `GET /v1/system/components`.
3. Sign in or create the first administrator using the one-time bootstrap token.
4. Confirm the instance list reflects PostgreSQL records, not sample rows.
5. Confirm logout revokes the session.

A successful Worker deployment alone is not proof that the API host, database, Tunnel, or Access policy is working.

## Security and implementation limits

- The Worker forwards only the API routes and methods listed in `worker/index.js`.
- The Worker requires HTTPS for the upstream origin and fails closed if the origin or Access Service Token is missing.
- API responses are marked `Cache-Control: no-store`; cookies and upstream CORS headers are not forwarded.
- Keep the API host private behind Cloudflare Tunnel and Access.
- The current API authentication is an initial local-development foundation. Rate limiting, full account recovery and management, production security review, and complete operational hardening remain pending.
- The instance registry stores metadata only. QEMU/KVM, containerd, storage providers, and node mTLS enrollment are not implemented yet.
