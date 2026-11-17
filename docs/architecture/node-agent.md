# CloudForge Node Agent

## Milestone 0.3 boundary

The first Node Agent release is a local, read-only host inventory executable. It reports a versioned JSON snapshot containing hostname, operating system, kernel, architecture, logical CPU count, memory, and mounted disk capacity.

Run it with:

```bash
cargo run -p cloudforge-node-agent
```

The output is informational. Available memory and disk space can change immediately and must not be treated as reserved scheduler capacity.

## Security boundary

This milestone deliberately does not:

- listen on a TCP or HTTP port;
- accept commands, scripts, or executable payloads;
- invoke a shell or spawn host-management processes;
- modify disks, network configuration, services, or virtual machines;
- require root or administrator privileges;
- send host inventory to a remote service.

The agent should run as an unprivileged operating-system user.

## Future control-plane protocol

Remote registration and heartbeats are blocked until service identity and authorization are implemented. The intended design is outbound-only agent communication over TLS, with a per-node identity, credential rotation/revocation, narrow endpoint authorization, replay protection, and auditable state changes. Mutual TLS is the preferred candidate for node-to-control-plane authentication.

The future protocol must use versioned, typed messages. It must never provide a general-purpose remote shell or arbitrary command execution interface.

## Inventory contract

The JSON document includes `schema_version` so future fields and incompatible changes can be managed explicitly. Disk entries describe mounted filesystems, not raw block devices or storage-pool health. CPU count and free memory are observations, not scheduler guarantees.

## Next gate

Before remote registration, implement identity issuance, secure credential storage, TLS validation, authorization, heartbeat expiry, and revocation. Before QEMU/KVM operations, introduce a separate privileged-operation boundary with explicit allowlisted actions and audit records.
