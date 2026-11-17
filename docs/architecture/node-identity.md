# Node identity and secure enrollment — milestone 0.4

## Current implementation

The Node Agent supports a local preflight command:

```bash
cargo run -p cloudforge-node-agent -- --check-identity
```

It requires these environment variables:

- `CLOUDFORGE_NODE_ID`: stable node UUID.
- `CLOUDFORGE_CONTROL_PLANE_URL`: HTTPS URL of the future control plane.
- `CLOUDFORGE_NODE_CLIENT_CERT`: path to the node client certificate in PEM format.
- `CLOUDFORGE_NODE_CLIENT_KEY`: path to the node private key in PEM format.
- `CLOUDFORGE_NODE_CA_CERT`: path to the CA certificate used to validate the control plane.

The preflight checks the UUID, requires HTTPS, rejects URL user-info, checks that certificate files are readable and contain expected PEM markers, and on Unix rejects private keys accessible by group or others (permissions must be owner-only, normally mode 0600). It prints only a safe summary and never prints key material.

## Important limitation

This command validates configuration shape and file permissions. It does **not** cryptographically validate certificate chains, establish a TLS connection, prove possession of the private key, enroll the node, or authenticate a heartbeat. A successful preflight is not proof that the node is trusted or connected.

The gateway does not yet expose node enrollment endpoints. No credentials should be issued to untrusted machines, and this preflight must not be treated as an enrollment mechanism.

## Protocol requirements before remote enrollment

1. Use a private CA or explicitly configured trust roots; never disable certificate validation.
2. Authenticate the node with mutual TLS and bind the certificate identity to the registered node UUID.
3. Require short-lived enrollment authorization and one-time enrollment challenges.
4. Support credential rotation, revocation, and emergency node quarantine.
5. Authorize each operation by node identity and narrowly scoped capability.
6. Reject stale/replayed heartbeats and record last-seen state.
7. Audit enrollment, credential changes, revocation, and state transitions.
8. Keep the agent outbound-only; do not expose a general-purpose command channel.

## Private key handling

Provision private keys through a trusted host-level process or secret manager. Keep key files readable only by the agent's dedicated operating-system account. Never commit certificates, private keys, enrollment tokens, or production secrets to the repository.

## Next implementation gate

The next milestone must implement actual TLS client construction and a server-side enrollment flow with certificate-to-node binding, authorization, persistence, revocation, and integration tests using test certificates. Only then can remote node registration and heartbeats be enabled.
