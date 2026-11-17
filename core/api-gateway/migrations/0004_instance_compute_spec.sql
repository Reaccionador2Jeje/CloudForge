-- Extend instance registry records with a provider-neutral VM specification.
-- These fields describe desired configuration only; no hypervisor is invoked.
ALTER TABLE instances
    ADD COLUMN IF NOT EXISTS vcpu INTEGER NOT NULL DEFAULT 1
        CHECK (vcpu BETWEEN 1 AND 64),
    ADD COLUMN IF NOT EXISTS memory_mib INTEGER NOT NULL DEFAULT 1024
        CHECK (memory_mib BETWEEN 256 AND 262144),
    ADD COLUMN IF NOT EXISTS image_ref VARCHAR(128);

ALTER TABLE instances
    ADD CONSTRAINT instances_image_ref_format
    CHECK (
        image_ref IS NULL
        OR image_ref ~ '^[a-zA-Z0-9][a-zA-Z0-9._:/-]{0,127}$'
    );
