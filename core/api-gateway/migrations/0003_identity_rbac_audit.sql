-- Identity, authorization, and audit foundations.
-- This migration creates data structures only; it does not enable login or grant privileges.

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(254) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name VARCHAR(80) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'suspended', 'disabled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (email = LOWER(BTRIM(email))),
    CHECK (CHAR_LENGTH(email) BETWEEN 3 AND 254),
    CHECK (CHAR_LENGTH(display_name) BETWEEN 1 AND 80),
    CHECK (password_hash <> '')
);

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(48) NOT NULL UNIQUE,
    description VARCHAR(240) NOT NULL DEFAULT '',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (name ~ '^[a-z][a-z0-9_-]{1,47}$')
);

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource VARCHAR(64) NOT NULL,
    action VARCHAR(32) NOT NULL,
    description VARCHAR(240) NOT NULL DEFAULT '',
    UNIQUE (resource, action),
    CHECK (resource ~ '^[a-z][a-z0-9_.-]{0,63}$'),
    CHECK (action ~ '^[a-z][a-z0-9_-]{0,31}$')
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash CHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    CHECK (token_hash ~ '^[0-9a-f]{64}$'),
    CHECK (expires_at > created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_active
    ON user_sessions (user_id, expires_at DESC)
    WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS audit_events (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_node_id UUID REFERENCES nodes(id) ON DELETE SET NULL,
    event_type VARCHAR(80) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id VARCHAR(128),
    outcome VARCHAR(16) NOT NULL
        CHECK (outcome IN ('success', 'denied', 'failure')),
    request_id UUID,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    CHECK (event_type ~ '^[a-z][a-z0-9_.-]{1,79}$'),
    CHECK (resource_type ~ '^[a-z][a-z0-9_.-]{1,63}$'),
    CHECK (jsonb_typeof(details) = 'object')
);

CREATE INDEX IF NOT EXISTS idx_audit_events_occurred_at
    ON audit_events (occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_actor_user
    ON audit_events (actor_user_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_resource
    ON audit_events (resource_type, resource_id, occurred_at DESC);

-- Prevent accidental mutation of audit history through ordinary application queries.
CREATE OR REPLACE FUNCTION cloudforge_reject_audit_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit_events is append-only';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS audit_events_immutable ON audit_events;
CREATE TRIGGER audit_events_immutable
    BEFORE UPDATE OR DELETE ON audit_events
    FOR EACH ROW EXECUTE FUNCTION cloudforge_reject_audit_mutation();

INSERT INTO roles (name, description, is_system)
VALUES
    ('platform_admin', 'Full platform administration; assignment requires a trusted bootstrap process.', TRUE),
    ('operator', 'Operate approved infrastructure resources.', TRUE),
    ('viewer', 'Read-only access to permitted platform resources.', TRUE)
ON CONFLICT (name) DO NOTHING;

INSERT INTO permissions (resource, action, description)
VALUES
    ('system', 'read', 'Read platform health and capability information.'),
    ('instances', 'read', 'Read instance metadata.'),
    ('instances', 'create', 'Create instance metadata records.'),
    ('instances', 'update', 'Update instance metadata.'),
    ('instances', 'delete', 'Delete instance metadata.'),
    ('nodes', 'read', 'Read registered node metadata.'),
    ('nodes', 'manage', 'Manage trusted node lifecycle.'),
    ('users', 'read', 'Read user accounts.'),
    ('users', 'manage', 'Manage user accounts and account status.'),
    ('roles', 'read', 'Read role and permission assignments.'),
    ('roles', 'manage', 'Assign roles and permissions.'),
    ('audit', 'read', 'Read audit event history.'),
    ('projects', 'read', 'Read project metadata.'),
    ('projects', 'manage', 'Manage project membership and settings.'),
    ('secrets', 'read', 'Read authorized secret references.'),
    ('secrets', 'manage', 'Create, rotate, and revoke secrets.')
ON CONFLICT (resource, action) DO NOTHING;

-- The platform administrator role receives the complete permission set.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r CROSS JOIN permissions p
WHERE r.name = 'platform_admin'
ON CONFLICT DO NOTHING;

-- Operators can administer ordinary workload metadata, but not identity,
-- permissions, secrets, or audit history.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r CROSS JOIN permissions p
WHERE r.name = 'operator'
  AND (p.resource, p.action) IN (
      ('system', 'read'),
      ('instances', 'read'),
      ('instances', 'create'),
      ('instances', 'update'),
      ('instances', 'delete'),
      ('nodes', 'read'),
      ('projects', 'read')
  )
ON CONFLICT DO NOTHING;

-- Viewers receive read-only visibility and cannot inspect secrets or audit
-- history until a separate policy explicitly grants those permissions.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r CROSS JOIN permissions p
WHERE r.name = 'viewer'
  AND p.action = 'read'
  AND p.resource IN ('system', 'instances', 'nodes', 'projects')
ON CONFLICT DO NOTHING;
