# CloudForge — Real Hosting Platform Charter

## Product direction

CloudForge is Atdev Dev's infrastructure and hosting platform, not a mock cloud
or a static demo. Its goal is to host real workloads on compute nodes operated
by Atdev Dev or explicitly trusted infrastructure partners.

"Host what you can imagine" describes the range of legitimate workloads the
platform aims to support. It does not mean unrestricted anonymous execution or
a promise that every runtime is available on day one.

## Workload families

The platform is designed to grow toward these workload types:

| Workload | Target execution model |
| --- | --- |
| Linux servers (Ubuntu, Debian, Alpine, and others) | QEMU/KVM virtual machines |
| Windows Server | QEMU/KVM virtual machines with valid licensing and verified media |
| Web sites and APIs | OCI containers and managed application runtimes |
| Databases and caches | Isolated containers or dedicated virtual machines |
| Game servers | Dedicated containers or virtual machines with network/port allocation |
| Bots and background workers | Sandboxed containers with resource limits |
| Development environments and remote terminals | Per-user isolated virtual machines or containers |
| Custom software | Supported OCI images or a documented VM image workflow |

Additional providers may be added through versioned provider contracts. A
catalog entry is not evidence that a provider is installed or operational.

## Platform architecture

- **Control plane:** identity, projects, permissions, workload APIs, quotas,
  scheduling, audit events, and lifecycle orchestration.
- **Data plane:** enrolled compute nodes running the selected VM or container
  provider.
- **Storage plane:** persistent volumes, snapshots, backups, and verified
  image artifacts.
- **Network plane:** tenant-isolated networks, ingress/egress policy, DNS,
  ports, and service routing.
- **Management plane:** web console, API, CLI, and later mobile clients.

The initial compute strategy is QEMU/KVM for full virtual machines and
containerd for OCI workloads. A workload must use the provider appropriate
to its isolation and operational needs.

## Product principles

1. No fake instances, sample metrics, pretend node connections, or fabricated
   deployment progress in production views.
2. Every resource state must come from persisted control-plane state or a
   verified provider observation.
3. A request is not "running" until the data plane confirms readiness.
4. Provider outages and unavailable capacity must be reported honestly.
5. Every workload belongs to an authenticated user/project and has explicit
   CPU, memory, storage, network, and lifecycle limits.
6. Node agents use authenticated, narrowly scoped typed operations; they do
   not expose an unrestricted host shell to tenants.
7. Secrets are delivered through scoped secret mechanisms and never embedded
   in frontend code, URLs, logs, or public repositories.
8. Public services require abuse prevention, rate limits, auditability,
   backups where applicable, and an incident response path.
9. Workloads must comply with applicable law, provider terms, software
   licenses, and CloudForge's acceptable-use policy. Malware distribution,
   credential theft, phishing, spam, botnets, and attacks against third-party
   systems are not supported use cases.
10. The platform must not advertise free, unlimited, or production-grade
    capacity until the infrastructure and operating budget actually exist.

## Current implementation truth (2026-11-18)

The repository currently has an Axum API Gateway, PostgreSQL-backed identity
and metadata foundations, a web console, an OS image metadata catalog, storage
quota/reservation metadata, and a local read-only Node Agent preflight.

The following production hosting capabilities are still blockers:

- Production deployment and hardening of the control plane.
- Trusted node enrollment, certificate lifecycle, and authenticated heartbeats.
- Project-level tenant isolation and authorization across all resources.
- A scheduler with real capacity reservations.
- Executable QEMU/KVM VM lifecycle operations.
- containerd workload lifecycle operations.
- Verified image ingestion and publishing.
- Physical persistent-volume allocation and backup/restore.
- Tenant-isolated networking, ingress, DNS, and firewall policy.
- Metrics, alerts, quotas, abuse response, and recovery testing.

The current instance, image, installation, and storage registries contain
metadata/plans. They do not by themselves create VMs, containers, disks, or
network endpoints. The current Node Agent preflight is local and read-only.

## Delivery roadmap

### Phase 1 — Make the control plane production-safe

- Finish production authentication hardening and account/session protections.
- Complete tenant/project ownership checks for every resource API.
- Implement service identity, node mTLS enrollment, certificate rotation,
  revocation, and replay protection.
- Add immutable audit coverage for infrastructure lifecycle operations.
- Add rate limits, quotas, policy checks, secret handling, and abuse controls.

### Phase 2 — Bring up the first real compute node

- Select and provision a dedicated Linux x86_64 host with hardware
  virtualization support.
- Install and validate QEMU/KVM and libvirt on that host.
- Enroll the node through authenticated mTLS; never expose the agent publicly.
- Implement a narrow typed node protocol and scheduler capacity reporting.
- Run integration tests on the dedicated host before accepting user workloads.

### Phase 3 — First real VM hosting

- Implement verified Ubuntu 24.04 image ingestion.
- Allocate real persistent virtual disks and isolated virtual networks.
- Implement create, start, stop, restart, inspect, and delete lifecycle calls.
- Report provider-observed state and test cleanup, crash recovery, and quotas.

### Phase 4 — Container and application hosting

- Integrate containerd through a typed provider interface.
- Support OCI images, health checks, restart policies, logs, ports, and
  resource limits.
- Add deployment workflows for web apps, APIs, databases, workers, and
  game-server workloads.
- Add TLS ingress, DNS, network isolation, and persistent storage attachments.

### Phase 5 — Operate a dependable hosting service

- Add monitoring, alerts, backups, restore drills, maintenance workflows,
  capacity planning, and incident response.
- Publish supported runtime versions, limits, availability expectations, and
  acceptable-use rules.
- Enable public signups and workload creation only after security and
  operational gates pass.

## Definition of real hosting

CloudForge may call a workload hosted only when all of the following are true:

- The request is authenticated and authorized for its owner/project.
- A trusted node and provider have accepted the typed operation.
- Capacity and storage have been reserved on actual infrastructure.
- The provider has created and started the workload.
- Health/readiness has been observed and persisted.
- Network access is configured according to policy.
- Stop/delete and recovery behavior have been tested.

Until then, the UI must label the feature as planned, unavailable, or
metadata-only rather than presenting it as a running server.
