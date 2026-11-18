# OpenWeb Linux terminal workload integration

## Status

Design and dependency record only. This document does not enable terminal
sessions, execute commands, create Ubuntu guests, or expose a shell endpoint.
OpenWeb Linux must remain explicit that its current browser shell is not a
real operating-system host until the execution path below is implemented and
tested.

## Target

OpenWeb Linux will use CloudForge as its control plane and a dedicated
Ubuntu 24.04 LTS guest as its data plane. The browser terminal is a client;
CloudForge must never execute tenant-provided shell text inside the API
Gateway or Node Agent process.

Preferred initial deployment is one isolated Ubuntu VM per authorized user.
A shared multi-tenant shell is out of scope for the first release.

## Required CloudForge capabilities

The current CloudForge repository does not yet provide the following
capabilities. They are hard blockers, not configuration steps:

- Trusted Node Agent enrollment and mTLS identity binding.
- Compute API, scheduler, and QEMU/KVM VM lifecycle execution.
- Verified Ubuntu 24.04 image ingestion and resolution.
- Persistent root-volume allocation and tenant-scoped storage quotas.
- Isolated networking and outbound-network policy.
- Resource quotas, ownership enforcement, and lifecycle audit records.
- A terminal-session broker with short-lived, single-use connection tickets.

The existing instance registry stores pending metadata only. The QEMU
preflight and VM plan commands are read-only/validation-only. They must not be
treated as a provisioned VM.

## Proposed lifecycle

1. An authenticated user requests an OpenWeb Linux environment through a
   tenant-scoped CloudForge API.
2. CloudForge validates authorization, quota, image readiness, and capacity.
3. The scheduler selects a trusted node and asks its narrowly scoped agent to
   create an Ubuntu 24.04 VM using typed provider operations.
4. CloudForge waits for guest health and registers the environment as ready.
5. The user requests a terminal session. CloudForge issues a short-lived,
   single-use ticket bound to the user, instance, origin, and expiry.
6. The browser opens a WebSocket to the terminal gateway and presents that
   ticket. The gateway validates it before attaching to the guest PTY.
7. Input/output is streamed over the WebSocket. Resize, disconnect, idle
   timeout, maximum session duration, and explicit termination are handled
   by the broker.
8. All lifecycle actions are audited. Terminal input/output is not logged by
   default because it may contain secrets.

## Proposed API contract

These routes are a target contract, not implemented endpoints:

- `POST /v1/instances` — request an Ubuntu 24.04 environment (must evolve
  from the current metadata-only registry into a real provisioner).
- `GET /v1/instances/{id}` — retrieve owner-scoped environment state.
- `POST /v1/instances/{id}/terminal-sessions` — issue a short-lived,
  single-use terminal ticket.
- `DELETE /v1/instances/{id}/terminal-sessions/{session_id}` — terminate a
  session.
- `wss://<terminal-origin>/v1/sessions/{session_id}` — attach to the PTY
  using the ticket during the WebSocket handshake.

The browser must never receive CloudForge service credentials, node-agent
credentials, database credentials, or long-lived infrastructure tokens.

## Security requirements

- No public unauthenticated shell and no shared administrator/root shell.
- One VM per user initially; never multiplex unrelated users into one Unix
  account or filesystem.
- Run the guest with bounded vCPU, RAM, disk, process count, and session count.
- Keep the hypervisor, libvirt socket, Docker/containerd socket, and host
  filesystem inaccessible to tenant sessions.
- Do not mount the host root filesystem or pass privileged devices into guests.
- Enforce owner checks on every instance and session operation.
- Use short-lived, single-use tickets; never put bearer tokens in URLs.
- Validate WebSocket Origin, authenticate before PTY attachment, cap message
  sizes, and rate-limit session creation.
- Restrict guest networking by policy and prevent access to host, metadata,
  control-plane, and other tenant networks.
- Apply guest security updates and support environment suspension, deletion,
  and recovery.
- Keep command output out of application logs by default.

## Delivery gates

1. Finish node trust and mTLS enrollment.
2. Implement and test QEMU/KVM lifecycle operations on a dedicated Linux host.
3. Add verified Ubuntu 24.04 image ingestion and provisioning.
4. Implement tenant ownership, quotas, storage, networking, and audit coverage.
5. Implement the terminal ticket broker and WebSocket PTY gateway.
6. Add an OpenWeb Linux browser adapter with explicit disconnected/connecting/
   connected states and no simulated fallback presented as a real host.
7. Run isolation, authorization, abuse, resource-exhaustion, and recovery tests
   before enabling public access.

Until these gates pass, OpenWeb Linux must describe the browser shell as a
local simulation or show the real-host feature as unavailable. A successful
static-site deployment does not imply that an Ubuntu host exists.
