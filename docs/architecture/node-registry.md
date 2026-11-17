# Node registry data model

## Purpose

Milestone 0.6 introduces the persistent data model for compute nodes. The
registry is a control-plane record; it is not proof that a machine is connected
or trusted.

The migration `0002_create_nodes.sql` creates a `nodes` table with:

- Stable UUID primary key and a human-readable name.
- Explicit lifecycle status: `pending`, `active`, `quarantined`, or `revoked`.
- Optional agent version and last authenticated heartbeat timestamp.
- Optional SHA-256 certificate fingerprint, unique when present.
- Optional JSONB inventory snapshot.
- Creation and update timestamps.

The certificate fingerprint is reserved for a future authenticated mTLS
implementation. It must never be populated from an unauthenticated request.
Likewise, `last_seen_at` must represent a successfully authenticated heartbeat,
not a client-supplied timestamp.

## Current security boundary

This migration does not add public node endpoints, enrollment tokens, certificate
issuance, or heartbeat handling. The Node Agent remains local and read-only.
Do not manually mark a node active merely because its inventory JSON was
received. Keep the API bound to loopback while API authentication and
authorization are not implemented.

## Required next steps

1. Implement a certificate authority/trust-root configuration and TLS server
   verification.
2. Add an authenticated enrollment workflow with one-time authorization,
   certificate-to-node UUID binding, and replay protection.
3. Persist only server-verified certificate fingerprints and authenticated
   heartbeats.
4. Add revocation/quarantine operations behind authenticated operator
   authorization and audit logging.
5. Add integration tests for invalid certificates, mismatched identities,
   replayed challenges, revoked nodes, and heartbeat expiry.

Until those gates pass, the table is schema groundwork only.

## Local read-only API view

The API Gateway exposes `GET /v1/nodes` when PostgreSQL is configured. It
returns node ID, name, lifecycle status, agent version, last authenticated
heartbeat timestamp, and creation time. It deliberately omits certificate
fingerprints and has no write operations.

This endpoint is part of the local development control plane, not a remote
multi-tenant API. Keep the gateway bound to loopback. Authentication and
authorization must be implemented before exposing node or instance data beyond
the trusted local host.
