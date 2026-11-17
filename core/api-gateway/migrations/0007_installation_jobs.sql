-- Persistent, auditable installation plans. These records never execute workloads.
CREATE TABLE IF NOT EXISTS installation_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id UUID NOT NULL REFERENCES instances(id) ON DELETE RESTRICT,
    profile_id VARCHAR(64) NOT NULL,
    installer_image_id UUID NOT NULL REFERENCES images(id) ON DELETE RESTRICT,
    status VARCHAR(16) NOT NULL DEFAULT 'planned'
        CHECK (status = 'planned'),
    execution_enabled BOOLEAN NOT NULL DEFAULT FALSE CHECK (execution_enabled = FALSE),
    blockers JSONB NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(blockers) = 'array'),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (profile_id IN ('windows-server-2025', 'windows-server-2022'))
);

CREATE INDEX IF NOT EXISTS idx_installation_jobs_instance_created
    ON installation_jobs (instance_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_installation_jobs_status_created
    ON installation_jobs (status, created_at DESC);

INSERT INTO permissions (resource, action, description)
VALUES
    ('installations', 'read', 'Read planned guest installation jobs.'),
    ('installations', 'create', 'Create non-executable guest installation plans.')
ON CONFLICT (resource, action) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.resource = 'installations'
WHERE r.name = 'platform_admin'
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.resource = 'installations' AND p.action IN ('read', 'create')
WHERE r.name = 'operator'
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.resource = 'installations' AND p.action = 'read'
WHERE r.name = 'viewer'
ON CONFLICT DO NOTHING;
