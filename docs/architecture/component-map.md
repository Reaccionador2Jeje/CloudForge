# CloudForge Component Map and Integration Order

This document is the implementation map for the platform. A catalog entry is
not a claim that its service is operational. The API component registry is the
source for current maturity labels.

## Control plane

- API Gateway: implemented Axum service.
- PostgreSQL: optional metadata persistence.
- Projects and tenancy: project ownership and tenant isolation.
- Identity and sessions: user authentication and session lifecycle.
- RBAC and policy engine: permission assignment and operation authorization.
- Compute API: provider-neutral workload definitions.
- Container API: provider-neutral container definitions.
- Storage API: provider-neutral volume and snapshot definitions.
- Network API and IPAM: network intent and address allocation.
- Scheduler and quotas: capacity placement and resource limits.
- Image Service and Image Builder: artifact catalog and reproducible image pipeline.
- Event Bus and Notifications: asynchronous coordination and operator messages.
- Usage and Billing: optional metering and cost allocation.

## Data plane

- Node Agent and Node Registry: host inventory and node metadata foundations.
- Node mTLS: authenticated node identity and heartbeat transport.
- QEMU/KVM: virtual machine provider.
- containerd: container provider.
- Storage providers: local ZFS first; distributed Ceph only after operational requirements are defined.
- Network providers: virtual switching, routing, isolation, and firewall integration.
- Load balancing and backup/restore: workload traffic and data resilience.

## Security and observability

- Secrets Management: encrypted secret storage and key rotation.
- Audit Log: append-only event history.
- Metrics, centralized logging, and alerting: operational visibility.
- API documentation and provider plugin contracts: stable integration surfaces.

## Implementation order

1. **Shared security foundations:** identity schema, RBAC, sessions, audit history,
   secret-handling design, and policy enforcement.
2. **Node trust:** real mTLS transport, certificate lifecycle, node identity
   binding, authenticated heartbeat, revocation, and replay protection.
3. **Provider-neutral APIs:** project ownership, quotas, compute/container/
   storage/network resource definitions, validation, and audit integration.
4. **Scheduling and artifacts:** capacity model, scheduler, image catalog,
   image verification, and event delivery.
5. **Infrastructure providers:** QEMU/KVM, containerd, storage, and networking.
6. **Operations:** backup/restore, metrics, logs, alerts, notifications, CLI,
   and mobile client.

The requested order intentionally postpones QEMU/KVM, containerd, and storage
providers until shared control-plane and security components are in place.
Provider work must include integration tests and must not be represented as
complete merely because a catalog entry or API shape exists.

## Current security foundations

Migration 0003_identity_rbac_audit.sql creates users, roles, permissions,
role assignments, hashed-session-token storage, and append-only audit event
storage. It inserts only role definitions; it creates no users, grants no user
roles, issues no sessions, and does not enable authentication or authorization.
The API therefore remains local-development-only and must not be exposed to
untrusted networks.
