# CloudForge API host deployment

This guide runs the Rust API Gateway and PostgreSQL on a Linux host with Docker Compose. The API is published only on host loopback (`127.0.0.1:8080`); do not open port 8080 in the host firewall or router. Cloudflare Tunnel and Access are configured separately.

## Requirements

- A supported 64-bit Linux server with Docker Engine and the Docker Compose plugin.
- A hostname in a Cloudflare-managed zone.
- A Cloudflare Tunnel and Access Service Auth policy for the API hostname.

Do not use an Android/Termux device as the production host. It is fine for editing the repository, but this deployment expects a persistent Linux Docker host.

## 1. Prepare secrets

Clone the repository and create a private environment file:

```bash
git clone https://github.com/Reaccionador2Jeje/CloudForge.git
cd CloudForge
cp .env.example .env
chmod 600 .env
```

Edit `.env` on the server. Set a unique, strong PostgreSQL password and a one-time bootstrap token generated on that server:

```bash
openssl rand -hex 32
```

Set the output as `CLOUDFORGE_BOOTSTRAP_TOKEN`. Never commit `.env`, paste secrets into chat, or place them in the frontend. The sample PostgreSQL password is for local development only and must be replaced.

## 2. Start the API and database

```bash
docker compose up -d --build
docker compose ps
docker compose logs --tail=100 api
```

The API container waits for PostgreSQL health, then applies database migrations at startup. Verify from the Linux host:

```bash
curl --fail http://127.0.0.1:8080/health/live
curl --fail http://127.0.0.1:8080/health/ready
```

The ready endpoint should report PostgreSQL as connected. Docker publishes the API only to host loopback.

## 3. Connect Cloudflare Tunnel

Create a Cloudflare Tunnel in the Cloudflare dashboard and install/run `cloudflared` on this same Linux host. Configure a public hostname such as `api.your-domain.example` with the tunnel service set to `http://127.0.0.1:8080`.

Use a dedicated API hostname. Do not create a public DNS record that points directly at the server IP, and do not expose port 8080 to the Internet.

## 4. Protect the hostname with Cloudflare Access

Create a self-hosted Access application for the API hostname. Add a **Service Auth** policy that allows only the Service Token intended for the CloudForge Worker. Do not use an allow-everyone policy.

Create an Access Service Token. In the `cloudforge` Worker settings configure:

- Variable `CLOUDFORGE_API_ORIGIN`: `https://api.your-domain.example`
- Secret `CF_ACCESS_CLIENT_ID`: the Access Service Token client ID
- Secret `CF_ACCESS_CLIENT_SECRET`: the Access Service Token client secret

These are not the Cloudflare API deployment token. Never store them in Git or browser JavaScript.

## 5. Verify the public console

After Tunnel, Access, and Worker settings are active, open the CloudForge console. It should load the real component catalog and health status. Create a regular account from the **Crear cuenta** form, then sign in using its email and password. For the first platform administrator, use the separate one-time bootstrap form. Keep the bootstrap token private and remove it from the server environment after the first administrator has been created; recreate the API service after changing environment values:

```bash
docker compose up -d --force-recreate api
```

## Operational notes

- Back up the PostgreSQL volume before upgrades or host maintenance.
- Keep Docker, the Linux host, and Cloudflare Tunnel updated.
- Monitor logs with `docker compose logs -f api`.
- Do not expose the API port publicly or bypass Access.
- The API authentication layer is still an early implementation. Rate limiting, account administration/recovery, deeper security review, and production hardening remain prerequisites for a broad public launch.
- This deployment does not implement QEMU/KVM, containerd workloads, storage providers, or node enrollment. Those remain separate milestones.
