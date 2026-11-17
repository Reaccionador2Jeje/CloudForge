-- Installation plans belong to the account that created them.
-- Existing legacy plans with no creator remain hidden from self-service accounts.
CREATE INDEX IF NOT EXISTS idx_installation_jobs_created_by
    ON installation_jobs (created_by, created_at DESC);
