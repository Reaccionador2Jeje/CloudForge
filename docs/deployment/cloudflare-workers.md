# CloudForge Console — Cloudflare Workers deployment

The public console is deployed as a **static-assets-only Cloudflare Worker** named `cloudforge`. Wrangler uploads the contents of `web/console/`; no Rust service, database, credentials, or node-agent interface is deployed with this site.

## Public demo behavior

- A persistent notice states that the API Gateway is not connected.
- Example instance rows are marked as simulated.
- Instance creation is disabled.
- Diagnostics do not send requests to API endpoints.
- No infrastructure health or capacity is represented as live data.

The Rust API Gateway remains a separate service. Do not point this public static console at an unauthenticated API. Before connecting a remote API, implement user authentication, authorization, CORS policy, rate limiting, and secure deployment configuration.

## Option A: Deploy from GitHub Actions (recommended for Android/Termux)

The repository includes a manual workflow at `.github/workflows/deploy.yml`. It runs Wrangler on a GitHub-hosted Linux runner, so Wrangler does not need to run on Android.

### 1. Create a Cloudflare API token

In the Cloudflare dashboard, create an API token with permission to edit Workers scripts for the Cloudflare account that owns the `cloudforge` Worker. Restrict the token to that account where possible. Copy the token once and keep it private.

Also obtain the Cloudflare **Account ID** from the account's dashboard overview.

### 2. Add GitHub repository secrets

Open the `Reaccionador2Jeje/CloudForge` repository:

1. Go to **Settings → Secrets and variables → Actions**.
2. Select **New repository secret**.
3. Add `CLOUDFLARE_API_TOKEN` with the API token.
4. Add `CLOUDFLARE_ACCOUNT_ID` with the account ID.

Never commit these values to the repository or paste them into issues, logs, or chat.

### 3. Run the deployment

1. Open the repository's **Actions** tab.
2. Select **Deploy CloudForge Console**.
3. Press **Run workflow**, keep branch `main`, and confirm.
4. Open the new run and wait for the deploy job to finish.
5. Copy the `workers.dev` URL from the successful deployment log.

This workflow is manual-only; ordinary pushes do not publish the public site.

## Option B: Deploy locally

Requirements:

- Node.js and npm (or a compatible Node.js package runner).
- A Cloudflare account authorized for Workers.
- Wrangler 4 on a supported platform.

From the repository root:

```bash
npx wrangler@4 deploy
```

For a local preview:

```bash
npx wrangler@4 dev
```

Wrangler reads `wrangler.jsonc` and publishes the static files from `web/console/` to the `cloudforge` Worker. With `workers_dev` enabled, Cloudflare assigns a URL under `workers.dev` on the account configured subdomain.

**Platform note:** Wrangler's local runtime dependency does not currently support Android ARM64. On Android/Termux, use Option A instead of trying to run Wrangler locally.

## Verify

After deployment, open the URL printed by Wrangler or shown in the GitHub Actions log and confirm:

1. The demo notice is visible.
2. The status says `Modo demo · API no conectada`.
3. The instances are clearly labelled as simulated.
4. Instance creation is disabled.
5. Diagnostics explicitly say no API request was sent.

A successful upload confirms publication of the static console only. It does not deploy or start the Rust API Gateway, PostgreSQL, QEMU/KVM, or Node Agents.
