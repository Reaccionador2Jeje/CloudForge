-- Node registry foundation. This table does not enable remote enrollment.
-- Certificate fingerprints are populated only by a future authenticated mTLS flow.
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY,
    name VARCHAR(63) NOT NULL,
    status VARCHAR(24) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'quarantined', 'revoked')),
    agent_version VARCHAR(64),
    certificate_fingerprint_sha256 CHAR(64) UNIQUE,
    inventory JSONB,
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (name = BTRIM(name)),
    CHECK (CHAR_LENGTH(name) BETWEEN 1 AND 63),
    CHECK (
        certificate_fingerprint_sha256 IS NULL
        OR certificate_fingerprint_sha256 ~ '^[0-9a-f]{64}$'
    )
);

CREATE INDEX IF NOT EXISTS idx_nodes_status
    ON nodes (status);

CREATE INDEX IF NOT EXISTS idx_nodes_last_seen_at
    ON nodes (last_seen_at DESC);
