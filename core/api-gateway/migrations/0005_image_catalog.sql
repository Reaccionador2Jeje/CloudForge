-- Versioned operating-system image metadata catalog.
-- Image records are metadata only; this migration does not download or trust artifacts.
CREATE TABLE IF NOT EXISTS images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(96) NOT NULL UNIQUE,
    name VARCHAR(120) NOT NULL,
    os_family VARCHAR(32) NOT NULL,
    os_version VARCHAR(32) NOT NULL,
    architecture VARCHAR(16) NOT NULL CHECK (architecture IN ('x86_64', 'aarch64')),
    disk_format VARCHAR(16) NOT NULL CHECK (disk_format IN ('qcow2', 'raw')),
    source_url TEXT,
    checksum_sha256 CHAR(64),
    status VARCHAR(16) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'verified', 'deprecated')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT images_slug_format CHECK (slug ~ '^[a-zA-Z0-9][a-zA-Z0-9._/-]{0,95}$'),
    CONSTRAINT images_checksum_format CHECK (
        checksum_sha256 IS NULL OR checksum_sha256 ~ '^[a-fA-F0-9]{64}$'
    ),
    CONSTRAINT images_verified_requires_checksum CHECK (
        status <> 'verified' OR checksum_sha256 IS NOT NULL
    )
);

CREATE INDEX IF NOT EXISTS idx_images_status_created_at
    ON images (status, created_at DESC);

INSERT INTO permissions (resource, action, description)
VALUES
    ('images', 'read', 'Read operating-system image catalog metadata.'),
    ('images', 'create', 'Register draft operating-system image metadata.')
ON CONFLICT (resource, action) DO NOTHING;

-- Only platform administrators can register image metadata.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.resource = 'images' AND p.action IN ('read', 'create')
WHERE r.name = 'platform_admin'
ON CONFLICT DO NOTHING;

-- Operators and viewers may inspect the catalog but cannot publish entries.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.resource = 'images' AND p.action = 'read'
WHERE r.name IN ('operator', 'viewer')
ON CONFLICT DO NOTHING;
