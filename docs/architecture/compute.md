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


## Operating-system image catalog

Migration `0005_image_catalog.sql` adds a versioned metadata registry. The
authenticated `GET /v1/images` endpoint lists entries; `POST /v1/images`
allows platform administrators to register draft entries. Operators and
viewers have read-only access.

Each entry records a stable slug, display name, OS family and version,
architecture (`x86_64` or `aarch64`), disk format (`qcow2` or `raw`),
optional HTTPS source URL, optional SHA-256 checksum, and lifecycle status.
New records are always `draft`; the database prevents a record from being
marked `verified` without a checksum.

This is metadata registration only. CloudForge does not fetch or verify the
source, validate image contents, sign artifacts, distribute images, or boot
them. The supplied URL and checksum are untrusted metadata until a future
trusted ingestion and verification pipeline exists. Instance creation does
not yet resolve `image_ref` against this catalog.


## Windows Server installation media

The image catalog distinguishes prepared disk images (`disk_image`, using `qcow2` or `raw`) from installer media (`installer_iso`, using `iso`). Migration `0006_image_installer_media.sql` seeds draft x86_64 entries for Windows Server 2025 and Windows Server 2022 without embedding or redistributing Microsoft installation files.

These are catalog placeholders, not ready-to-boot cloud images. The future Windows guest workflow must obtain media through authorized Microsoft channels, verify provenance and checksums through a trusted ingestion process, test supported virtual firmware and devices, provide required storage/network drivers (including a supported VirtIO driver path where applicable), and enforce a valid Windows Server licensing/activation model. CloudForge does not provide product keys, activation bypasses, or license rights.


## Windows Server guest installation profiles

The read-only endpoint `GET /v1/compute/guest-profiles` publishes planned profiles for Windows Server 2025 and 2022. Each profile describes the intended x86_64 guest baseline: UEFI firmware, Q35 machine type, ISO installer media, VirtIO SCSI storage, and VirtIO network devices. These are proposed integration defaults, not tested production settings. Secure Boot and TPM behavior are explicitly marked as requiring validation.

The profile includes an installation-stage outline: validate authorized media, allocate resources, prepare virtual hardware, attach the ISO, boot Windows Setup, load storage/network drivers, complete installation, verify guest health and licensing, then produce a reusable image. `execution_enabled` is false and `readiness` is `planned`; the endpoint does not create disks, start QEMU, install Windows, fetch ISOs, or activate licenses.

Before enabling execution, CloudForge still needs a trusted node agent and scheduler, tenant-scoped authorization, storage/network providers, safe media ingestion and checksum verification, driver provenance controls, firmware/device compatibility tests, and a compliant Windows Server licensing/activation process.


## Installation plan registry

The authenticated API exposes a persistent, non-executable installation-plan registry:

- `GET /v1/compute/installations` lists plans.
- `GET /v1/compute/installations/{id}` retrieves one plan.
- `POST /v1/compute/installations` creates a plan for an existing instance, a supported Windows Server profile, and a matching installer ISO catalog entry.

Example request:

```json
{
  "instance_id": "EXISTING-INSTANCE-UUID",
  "profile_id": "windows-server-2025",
  "installer_image_slug": "windows-server/2025"
}
```

New plans are stored with status `planned`, `execution_enabled: false`, and explicit blocker identifiers. Current blockers include the unavailable QEMU/KVM provider, unvalidated virtual hardware, missing VirtIO driver delivery, and the absent Windows Server licensing workflow. Draft installer media adds an `installer_media_not_verified` blocker. The API checks that the selected catalog entry matches the requested Windows Server version and is an x86_64 installer ISO.

The registry records intent; it does not reserve resources, download media, create disks, start a VM, run Windows Setup, or claim installation progress. The database constraint prevents `execution_enabled` from being set to true in this milestone. A trusted worker, scheduler, tenant ownership checks, stage/event history, cancellation policy, and real QEMU/KVM integration are required before execution can be enabled.


## First QEMU/KVM host adapter milestone

The Node Agent now has a local, read-only QEMU/libvirt preflight command:

```bash
cloudforge-node-agent --compute-preflight
```

It checks that the host is Linux, that `/dev/kvm` exists, that
`qemu-system-x86_64` and `virsh` are available, and that the local
`qemu:///system` libvirt connection responds. When libvirt is reachable, it reads each defined domain's UUID, name, and
current libvirt state (`running`, `shut off`, and so on). The inventory is
collected through fixed `virsh list --all --uuid`, `domname`, and `domstate`
queries. UUID values are validated before use as process arguments. The command emits a versioned JSON report (schema version 2), including a
`domain_inventory_available` flag and a `domains` array containing `uuid`,
`name`, and `state`. It returns exit code 2 when any prerequisite or the
read-only domain inventory is unavailable.

This is an initial host-side adapter, not a provisioner. It invokes only
fixed, read-only commands without a shell and accepts no user-provided command
arguments. It does not create, start, stop, delete, or modify VMs; allocate
storage; change host networking; or expose libvirt to the network.

### Linux host prerequisites

On a supported Linux compute node, install the distribution packages for:

- QEMU system emulator for x86_64.
- KVM kernel support and access to `/dev/kvm`.
- libvirt daemon and `virsh` client.
- A running system libvirt service and an authorized CloudForge agent account.

The agent must be run with the minimum local privileges required to access
`/dev/kvm` and the system libvirt socket. Do not expose the libvirt socket or
grant the agent unrestricted administrative access over the network.

The current agent has no authenticated connection to the API Gateway. The
preflight is local-only and must not be interpreted as a registered or trusted
compute node. VM lifecycle operations remain disabled until node mTLS,
enrollment, tenant ownership, scheduler reservations, managed storage, and
network isolation are implemented and tested.

## Typed libvirt domain definition builder

The Node Agent exposes a validation-only XML renderer:

```bash
cloudforge-node-agent --render-domain-xml <name> <vcpu> <memory-mib> <disk-path>
```

It validates a conservative domain name, CPU and memory bounds, and a single
QCOW2 filename directly inside
`/var/lib/cloudforge/pools/default/`. It renders a Q35 x86_64 KVM domain
definition with a VirtIO-SCSI controller. The command prints XML only; it
does not create the disk, call `virsh define`, start QEMU, or modify libvirt.
The managed disk pool must be provisioned separately by a future storage
provider. This renderer is a tested definition-building primitive, not a
production-ready VM provisioner.

## QCOW2 storage planning

The Node Agent can produce a deterministic, validation-only disk plan using
`storage::DiskRequest`. Plans use the managed default pool, QCOW2 format,
sparse allocation, and a 1–2048 GiB virtual size. The planner does not create
directories or files and does not invoke `qemu-img`; execution remains disabled
until a storage provider enforces filesystem ownership, quotas, free-space
checks, and safe lifecycle handling.


## Composed VM provisioning plan

The Node Agent now composes the QCOW2 disk planner and typed libvirt XML
renderer into one deterministic VM plan:

```bash
cloudforge-node-agent --plan-vm <name> <vcpu> <memory-mib> <disk-name> <disk-size-gib>
```

The JSON output contains the validated disk plan, the matching libvirt domain
XML, an explicit `execution_enabled: false` flag, and machine-readable
blockers for the missing storage provider, trusted node enrollment, scheduler
and capacity reservations, network isolation, and VM lifecycle executor.
The disk path in the XML is taken directly from the validated disk plan, so
the two artifacts cannot silently target different disk locations.

This command is a planning and validation operation only. It does not create
the QCOW2 file, create directories, invoke `qemu-img`, define a libvirt
domain, launch QEMU, or change host state. Do not enable execution until the
listed infrastructure and security gates are implemented and tested on a
dedicated Linux/KVM host.


## Read-only storage pool preflight

The Node Agent exposes a read-only storage readiness report:

```bash
cloudforge-node-agent --storage-preflight
```

It inspects the fixed default pool path
`/var/lib/cloudforge/pools/default`, reports whether it exists and is a
directory, checks for symlinks in the path, reports its permission mode and
owner UID, and checks whether `qemu-img --version` succeeds. The command
returns a versioned JSON report and exit code 2 when a prerequisite is
missing.

The preflight does not create the pool, write test files, change permissions,
inspect or modify disk contents, or create QCOW2 images. Permission mode bits
are only a preliminary signal: effective access, service identity, free space,
filesystem quotas, mount properties, and safe ownership policy still require
dedicated validation before a storage provider can be enabled.
