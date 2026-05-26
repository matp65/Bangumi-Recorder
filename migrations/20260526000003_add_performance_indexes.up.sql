-- Index for incremental sync: WHERE updated_at > ?
CREATE INDEX idx_recordings_updated_at ON recordings (updated_at);

-- Composite index for common queries: WHERE user_id = ? AND is_delete = 0
CREATE INDEX idx_recordings_user_delete ON recordings (user_id, is_delete);
