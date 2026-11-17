# CloudForge Architecture Overview

## Goal

CloudForge is an open-source infrastructure platform developed by Atdev Dev. It is intended to manage Atdev-operated physical infrastructure and expose cloud capabilities through consistent APIs and cross-platform clients.

The initial implementation is deliberately small. The repository will grow through independently documented services rather than starting as a distributed system before it is needed.

## High-level components

- **API Gateway:** authenticated entry point for client and service requests.
- **Identity:** users, service identities, roles, and authorization policies.
- **Compute:** virtual machine lifecycle and container workloads.
- **Storage:** volumes, images, snapshots, and backups.
- **Networking:** virtual networks, addresses, routing, and firewall policy.
- **Scheduler:** placement decisions based on capacity and policy.
- **Image Service:** catalog and lifecycle of bootable images.
- **Node Agent:** authenticated operations on registered physical hosts.
- **Clients:** web, desktop, and mobile management interfaces.

## Initial implementation boundary

Milestone 0.1 establishes the Rust workspace and a small Axum API Gateway with process health endpoints. It does not provision virtual machines, control host networking, or expose privileged host operations.

## Control plane and data plane

The **control plane** stores desired state, validates user intent, schedules work, and records operation status.

The **data plane** consists of physical nodes and the workloads running on them. Node agents will receive authenticated, narrowly scoped instructions from the control plane.

A node agent must not expose an unrestricted shell or accept arbitrary commands from remote clients. Privileged operations will be explicit, validated, auditable, and limited to the node's assigned responsibilities.

## Initial technology choices

| Area | Initial choice |
| --- | --- |
| Core services | Rust |
| HTTP API | Axum |
| Async runtime | Tokio |
| Logging | tracing |
| Virtualization | QEMU/KVM |
| Relational database | PostgreSQL |
| Internal messaging | To be evaluated when asynchronous service communication is introduced |
| Container runtime | containerd |
| Storage | Start with local development storage; evaluate ZFS and Ceph for deployment requirements |
| Web client | TypeScript |
| Mobile client | Flutter |

Technology choices may be revised through Architecture Decision Records (ADRs).

## Development principles

1. Secure defaults and least privilege.
2. Small, testable service boundaries.
3. Explicit API and data contracts.
4. Idempotent infrastructure operations.
5. Observable state transitions and auditable actions.
6. No assumption that a development machine is production infrastructure.
7. Open development and accessible client software.

## Planned next steps

1. Add shared configuration, error, and observability conventions.
2. Add PostgreSQL migrations and persistence boundaries.
3. Define the first control-plane resource model.
4. Design the authenticated node-agent protocol.
5. Integrate QEMU/KVM in an isolated development environment before enabling host-level operations.
