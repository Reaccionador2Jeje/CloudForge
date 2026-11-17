-- Distinguish install media from prepared virtual disk images.
ALTER TABLE images
    ADD COLUMN IF NOT EXISTS artifact_type VARCHAR(24) NOT NULL DEFAULT 'disk_image';

ALTER TABLE images
    DROP CONSTRAINT IF EXISTS images_disk_format_check;

ALTER TABLE images
    ADD CONSTRAINT images_disk_format_check
    CHECK (
        (artifact_type = 'disk_image' AND disk_format IN ('qcow2', 'raw'))
        OR (artifact_type = 'installer_iso' AND disk_format = 'iso')
    );

ALTER TABLE images
    ADD CONSTRAINT images_artifact_type_check
    CHECK (artifact_type IN ('disk_image', 'installer_iso'));

-- Windows installer media are catalog placeholders only.
-- No Microsoft ISO is downloaded, mirrored, or redistributed by this migration.
INSERT INTO images (
    slug, name, os_family, os_version, architecture,
    disk_format, artifact_type, source_url, checksum_sha256, status
)
VALUES
    ('windows-server/2025', 'Windows Server 2025', 'windows-server', '2025',
     'x86_64', 'iso', 'installer_iso', NULL, NULL, 'draft'),
    ('windows-server/2022', 'Windows Server 2022', 'windows-server', '2022',
     'x86_64', 'iso', 'installer_iso', NULL, NULL, 'draft')
ON CONFLICT (slug) DO NOTHING;
