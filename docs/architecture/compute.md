# Compute control-plane foundation

## Goal

CloudForge targets the IaaS market: operators should be able to provision and
manage virtual machines on physical infrastructure they own. The compute
control plane must remain independent from a specific hypervisor so additional
providers can be integrated without changing tenant-facing resource concepts.

## Current milestone

The API exposes a read-only provider catalog at `GET /v1/compute/providers`.
It describes the QEMU/KVM target and its prerequisites. It does not execute
QEMU, inspect host devices, allocate resources, create disks, configure
networking, or start virtual machines.

QEMU/KVM is deliberately reported as `planned` until the complete execution
path exists and is tested on a supported Linux host.

## Planned architecture

1. **Compute API:** validate tenant requests and maintain desired VM state.
2. **Scheduler:** select an eligible node using available CPU, memory, and
   policy reservations.
3. **Node trust:** authenticate the node using mutual TLS and bind its
   certificate to a stable node identity.
4. **QEMU provider:** translate a validated VM specification into a
   hypervisor-specific launch plan.
5. **Storage provider:** allocate and attach durable disks before boot.
6. **Network provider:** create isolated interfaces and apply network policy.
7. **Reconciliation:** compare desired and observed state and recover from
   interrupted operations.
8. **Audit and metering:** record lifecycle operations and resource usage.

## Initial VM resource model

The first tenant-facing VM specification will include:

- Stable resource UUID and project ownership.
- Human-readable name and lifecycle state.
- vCPU count and memory in MiB.
- Versioned operating-system image reference.
- Root-volume reference and optional additional volume references.
- Network-interface references.
- Creation timestamps and provider/node placement metadata.

The API must reject invalid resource sizes, unknown images, unauthorized
projects, and requests exceeding quotas before any host operation occurs.

## QEMU/KVM prerequisites

A production QEMU provider requires a supported Linux host with QEMU system
binaries, usable KVM access, a trusted CloudForge node agent, a scheduler,
durable storage, and a network isolation provider. The API Gateway must not
directly execute arbitrary shell commands or accept user-supplied QEMU command
lines. Provider operations will use typed, validated requests and narrowly
scoped node-agent actions.

## Delivery gates

- [x] Provider catalog and explicit prerequisite model.
- [ ] Versioned VM specification and validation.
- [ ] Tenant/project ownership and compute permissions.
- [ ] Trusted node enrollment and authenticated command protocol.
- [ ] Scheduler capacity reservations and placement.
- [ ] QEMU process lifecycle adapter on a dedicated Linux/KVM test host.
- [ ] Persistent disk and isolated network integration.
- [ ] End-to-end create, stop, restart, delete, and recovery tests.

No virtual machine is provisioned by the current milestone.

## Implemented VM specification registry

The existing authenticated `/v1/instances` registry now stores the first
provider-neutral VM request fields:

- `name`: required, 1–63 UTF-8 bytes after trimming.
- `vcpu`: optional, defaults to 1, accepted range 1–64.
- `memory_mib`: optional, defaults to 1024 MiB, accepted range 256–262144.
- `image_ref`: optional catalog-style identifier, up to 128 characters.

Migration `0004_instance_compute_spec.sql` adds the persisted fields and
database constraints. The existing RBAC-protected `GET /v1/instances` and
`POST /v1/instances` routes expose this metadata. The optional image reference
is not resolved against an image catalog yet.

Example request:

```json
{
  "name": "my-server",
  "vcpu": 2,
  "memory_mib": 2048,
  "image_ref": "ubuntu/24.04"
}
```

The API validates and stores a desired configuration only. It does not reserve
capacity, select a node, download an image, create a disk, configure a network,
or start a virtual machine.
