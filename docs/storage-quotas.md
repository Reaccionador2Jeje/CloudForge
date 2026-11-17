# Storage quotas, reservations, and volume plans

CloudForge provides account-level quota accounting and **metadata-only volume plans**. These APIs do not provision physical disks.

## API

All endpoints require a valid bearer session and the `storage` permission.

| Method | Endpoint | Purpose |
|---|---|---|
| GET | `/v1/storage/quota` | Read the authenticated account's quota and usage |
| GET | `/v1/storage/reservations` | List the latest 100 reservations belonging to the account |
| POST | `/v1/storage/reservations` | Reserve quota capacity |
| DELETE | `/v1/storage/reservations/{id}` | Release an active reservation owned by the account |
| GET | `/v1/storage/volumes` | List the latest 100 volume metadata records |
| POST | `/v1/storage/volumes` | Convert an active reservation into a planned volume record |
| DELETE | `/v1/storage/volumes/{id}` | Delete a planned/failed metadata record |

### Create a reservation

Send a JSON body containing `requested_gib` (integer from 1 through 2048) and an `Idempotency-Key` header (8–128 ASCII letters, digits, or `-_.:` characters).

```http
POST /v1/storage/reservations
Authorization: Bearer <session-token>
Content-Type: application/json
Idempotency-Key: request-unique-2026-11

{"requested_gib": 10}
```

A successful new reservation returns HTTP 201. Repeating the same key with the same size while the reservation is active returns the existing reservation with HTTP 200. Reusing the key with a different size or an inactive reservation returns HTTP 409.

Reservations expire after 30 minutes. A caller may release a reservation early with DELETE; release is idempotent for an already inactive reservation.

### Create a planned volume

A volume consumes one active reservation. The requested volume name must start with a lowercase ASCII letter and contain at most 48 lowercase letters, digits, hyphens, or underscores. Supported metadata formats are `qcow2` and `raw`.

```http
POST /v1/storage/volumes
Authorization: Bearer <session-token>
Content-Type: application/json

{
  "reservation_id": "00000000-0000-4000-8000-000000000000",
  "name": "project-root",
  "format": "qcow2"
}
```

The volume size is copied from the reservation; clients cannot change it during conversion. A new record returns HTTP 201. Retrying the same request for a consumed reservation returns the existing matching volume with HTTP 200. Trying to reuse that reservation for a different volume returns HTTP 409.

Deleting a volume record is permitted only while it is in `planned` or `failed` state. The API refuses to delete records in provider-managed states. Deleting a plan only removes its metadata from quota accounting; there is no physical disk to remove.

## Quota accounting

New accounts receive a default quota of 100 GiB. The migration seeds existing accounts with the same default. The quota endpoint reports:

- `quota_gib`: configured account quota.
- `used_gib`: volume records in planned, provisioning, ready, or deleting states.
- `reserved_gib`: unexpired reservations still in reserved state.
- `available_gib`: quota minus used and reserved capacity, clamped at zero.

Reservation and volume creation serialize quota checks per account by locking its quota row in PostgreSQL. Converting a reservation to a planned volume is one transaction: the volume is inserted and the reservation is marked consumed. Planned volumes count against quota, so the conversion cannot temporarily free capacity.

## Current limitations

- Reservations and volumes are PostgreSQL accounting/metadata records only.
- No storage directory or disk image is created.
- No `qemu-img` command is executed by the API.
- No node capacity is reserved and no storage provider is selected.
- No ZFS, Ceph, replication, snapshots, backup, or restore operation is implemented.
- The Node Agent storage preflight and allocation planners are local read-only/planning tools; they are not connected to a remote provider.
- The console can reserve quota, create volume metadata plans, and delete eligible plans. It does not provision or delete real volumes.

Physical storage provisioning must remain disabled until a trusted node provider, capacity reservation, safe path handling, storage lifecycle executor, recovery behavior, and integration tests are implemented.
