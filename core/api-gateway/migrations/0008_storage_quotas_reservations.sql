-- Persistent per-user storage quotas and idempotent reservations.
-- Reservations are accounting records only; they do not allocate host files.
CREATE TABLE IF NOT EXISTS storage_quotas (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    quota_gib BIGINT NOT NULL DEFAULT 100 CHECK (quota_gib BETWEEN 1 AND 1048576),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS storage_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    idempotency_key VARCHAR(128) NOT NULL,
    requested_gib BIGINT NOT NULL CHECK (requested_gib BETWEEN 1 AND 2048),
    status VARCHAR(16) NOT NULL DEFAULT 'reserved'
        CHECK (status IN ('reserved', 'released', 'expired', 'consumed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    UNIQUE (user_id, idempotency_key),
    CHECK (expires_at > created_at),
    CHECK (released_at IS NULL OR released_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_storage_reservations_user_active
    ON storage_reservations (user_id, expires_at)
    WHERE status = 'reserved';

-- The volume registry is metadata only. A volume becomes 'ready' only after a
-- future trusted storage provider confirms successful allocation.
CREATE TABLE IF NOT EXISTS storage_volumes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    reservation_id UUID UNIQUE REFERENCES storage_reservations(id) ON DELETE RESTRICT,
    name VARCHAR(48) NOT NULL,
    size_gib BIGINT NOT NULL CHECK (size_gib BETWEEN 1 AND 2048),
    format VARCHAR(16) NOT NULL DEFAULT 'qcow2' CHECK (format IN ('qcow2', 'raw')),
    status VARCHAR(16) NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'provisioning', 'ready', 'failed', 'deleting', 'deleted')),
    provider_volume_ref VARCHAR(256),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, name),
    CHECK (name ~ '^[a-z][a-z0-9_-]{0,47}$'),
    CHECK (
        (status IN ('planned', 'failed', 'deleted') AND provider_volume_ref IS NULL)
        OR status IN ('provisioning', 'ready', 'deleting')
    )
);

CREATE INDEX IF NOT EXISTS idx_storage_volumes_user_usage
    ON storage_volumes (user_id, status)
    WHERE status IN ('provisioning', 'ready', 'deleting');

-- Give existing and newly bootstrapped accounts the conservative default quota.
INSERT INTO storage_quotas (user_id)
SELECT id FROM users
ON CONFLICT (user_id) DO NOTHING;

CREATE OR REPLACE FUNCTION cloudforge_create_default_storage_quota()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO storage_quotas (user_id) VALUES (NEW.id)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS users_create_storage_quota ON users;
CREATE TRIGGER users_create_storage_quota
    AFTER INSERT ON users
    FOR EACH ROW EXECUTE FUNCTION cloudforge_create_default_storage_quota();

INSERT INTO permissions (resource, action, description)
VALUES
    ('storage', 'read', 'Read personal storage quota and reservations.'),
    ('storage', 'create', 'Create idempotent storage capacity reservations.'),
    ('storage', 'manage', 'Release personal storage reservations.')
ON CONFLICT (resource, action) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.resource = 'storage'
WHERE r.name IN ('platform_admin', 'operator', 'viewer')
  AND (
      (r.name = 'platform_admin')
      OR (r.name = 'operator' AND p.action IN ('read', 'create', 'manage'))
      OR (r.name = 'viewer' AND p.action = 'read')
  )
ON CONFLICT DO NOTHING;
