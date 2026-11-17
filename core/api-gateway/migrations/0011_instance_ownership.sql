-- Ensure self-service accounts can only see their own instance metadata.
ALTER TABLE instances
    ADD COLUMN IF NOT EXISTS owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_instances_owner_created
    ON instances (owner_user_id, created_at DESC);
