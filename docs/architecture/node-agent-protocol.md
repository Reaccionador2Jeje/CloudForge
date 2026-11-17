# Node Agent communication protocol (design draft)

## Status

This document defines a proposed protocol contract for a future authenticated
Node Agent connection. It is a design specification only. CloudForge does not
currently implement this protocol, establish mTLS sessions, enroll nodes, or
send heartbeats.

## Goals

- Authenticate each node with a client certificate issued by a CloudForge
  operator-controlled certificate authority.
- Bind the verified certificate identity to exactly one registered node UUID.
- Send bounded, versioned inventory snapshots and heartbeat messages.
- Keep node management operations separate from routine agent reporting.
- Reject stale, replayed, malformed, oversized, or unauthorized messages.

## Transport and trust

The future transport must use HTTPS with mutual TLS (mTLS):

1. The agent validates the control plane certificate chain against the
   configured CA and validates the server name.
2. The gateway validates the client certificate chain against its configured
   node CA, checks validity dates and revocation state, and extracts the
   authenticated node identity from a defined certificate identity field.
3. The identity extracted from the certificate must match the node UUID in
   the request. A request body must never be allowed to choose or override the
   authenticated identity.
4. The gateway computes the SHA-256 certificate fingerprint itself. It must
   never trust a fingerprint supplied by the agent.
5. TLS configuration must fail closed. No insecure certificate-verification
   bypass, plaintext fallback, or self-signed trust-on-first-use mode is allowed.

A successful local configuration preflight is not an authenticated session.
The current `--check-identity` command only checks configured paths, PEM
markers, and private-key file permissions; it does not parse certificates,
verify key/certificate matching, validate trust chains, or connect to a server.

## Proposed message envelope

Future agent messages should use a versioned JSON envelope with these fields:

| Field | Meaning |
| --- | --- |
| `protocol_version` | Integer protocol version; initially `1`. |
| `message_id` | UUID unique to one message, used for idempotency/replay handling. |
| `node_id` | Claimed node UUID; must match the certificate identity. |
| `agent_version` | Agent build version. |
| `inventory_schema_version` | Version of the inventory payload schema. |
| `inventory` | Bounded inventory snapshot, when the message includes inventory. |

The server records receipt time itself. Client-provided timestamps must not
control `last_seen_at`, node status, certificate validity, or authorization.

## Proposed operations

- **Heartbeat:** confirms that an already enrolled, non-revoked node is alive.
- **Inventory update:** replaces the latest accepted inventory snapshot after
  schema and size validation.
- **Enrollment:** a separate operator-authorized workflow. It must not be
  implemented as an unauthenticated first heartbeat.

The initial implementation should expose no remote shell, arbitrary command
execution, file transfer, port forwarding, or agent-driven node activation.

## Validation requirements

The gateway must reject messages when any of these conditions applies:

- Unsupported protocol or inventory schema version.
- Certificate chain, validity, identity binding, or revocation check fails.
- Node is unknown, pending without an approved enrollment, quarantined, or
  revoked.
- Message ID has already been accepted within the replay/idempotency window.
- JSON body, inventory arrays, strings, or disk entries exceed documented
  size/count limits.
- Numeric inventory values are inconsistent (for example, available memory
   exceeds total memory).
- Required fields are absent or unknown fields violate the selected schema
  policy.

Errors must not disclose private certificate details, filesystem paths, or
internal trust configuration.

## Rollout gates

Before enabling any network connection from the agent:

1. Implement and test CA loading, server-name verification, client certificate
   validation, and certificate/key matching.
2. Implement enrollment authorization, node identity binding, certificate
   issuance, rotation, and revocation.
3. Add authenticated heartbeat and inventory endpoints with strict request
   limits and replay/idempotency handling.
4. Add audit events for enrollment, certificate changes, quarantine, and
   revocation.
5. Add integration tests for invalid, expired, mismatched, revoked, and replayed
   credentials/messages.
6. Add authentication and authorization to the API Gateway before exposing it
   beyond loopback.

Until all gates pass, the Node Agent remains a local read-only inventory tool
and the node registry remains a local control-plane record.
