# Storage quotas and reservations

CloudForge currently provides **accounting reservations**, not physical disk provisioning.

## API

All endpoints below require a valid bearer session and the `storage` permission.

| Method | Endpoint | Purpose |
|---|---|---|
| GET | `/v1/storage/quota` | Read the authenticated account's quota and usage |
| GET | `/v1/storage/reservations` | List the latest 100 reservations belonging to the account |
| POST | `/v1/storage/reservations` | Reserve quota capacity |
| DELETE | `/v1/storage/reservations/{id}` | Release an active reservation owned by the account |

### Create a reservation

Send a JSON body containing `requested_gib` (integer from 1 through 2048) and an `Idempotency-Key` header (8–128 ASCII letters, digits, or `-_.:` characters).

```http
POST /v1/storage/reservations
Authorization: Bearer <session-token>
Content-Type: application/json
Idempotency-Key: request-unique-2026-01

{"requested_gib": 10}
```

A successful new reservation returns HTTP 201. Repeating the same key with the same size while the reservation is active returns the existing reservation with HTTP 200. Reusing the key with a different size or an inactive reservation returns HTTP 409.

Reservations expire after 30 minutes. A caller may release a reservation early with DELETE; release is idempotent for an already inactive reservation.

## Quota accounting

New accounts receive a default quota of 100 GiB. The current migration seeds existing accounts with the same default. The quota endpoint reports:

- `quota_gib`: configured account quota.
- `used_gib`: registered volumes in provisioning, ready, or deleting states.
- `reserved_gib`: unexpired reservations still in reserved state.
- `available_gib`: quota minus used and reserved capacity, clamped at zero.

Reservation creation serializes quota checks per account by locking its quota row in PostgreSQL. This prevents two concurrent requests for the same account from both spending the same remaining quota.

## Current limitations

- A reservation is a database accounting record only.
- No storage directory or disk image is created.
- No `qemu-img` command is executed by the API.
- No node capacity is reserved and no storage provider is selected.
- The `storage_volumes` table is a metadata foundation for a future trusted storage provider.
- The console displays reservations and quota, but does not provision or delete real volumes.

Physical storage provisioning must remain disabled until a trusted node provider, capacity reservation, lifecycle executor, and recovery behavior are implemented and tested.
