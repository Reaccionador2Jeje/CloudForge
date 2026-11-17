-- Planned volumes consume quota even before a provider allocates physical storage.
-- This prevents a consumed reservation from temporarily freeing account quota.
DROP INDEX IF EXISTS idx_storage_volumes_user_usage;
CREATE INDEX idx_storage_volumes_user_usage
    ON storage_volumes (user_id, status)
    WHERE status IN ('planned', 'provisioning', 'ready', 'deleting');
