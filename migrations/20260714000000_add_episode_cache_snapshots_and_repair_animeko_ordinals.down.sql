-- The repaired episode ordinals are intentionally not reverted: restoring known-bad
-- season-relative ordinals would corrupt user data again.
DROP TABLE IF EXISTS _br_migration_20260714_episode_ordinal_repairs;

DROP TRIGGER IF EXISTS episode_records_sync_update;
CREATE TRIGGER episode_records_sync_update AFTER UPDATE ON episode_records
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, ordinal, changed_at)
SELECT r.user_id, b.external_id, 'episode', NEW.ordinal, NEW.updated_at
FROM recordings r JOIN bangumi_info_easy b ON b.id = r.bangumi_id
WHERE r.id = NEW.recording_id
  AND NOT (
      OLD.watched = NEW.watched
      AND OLD.progress_seconds <=> NEW.progress_seconds
      AND OLD.duration_seconds <=> NEW.duration_seconds
      AND OLD.completed_at <=> NEW.completed_at
  );

ALTER TABLE bangumi_episodes
    DROP INDEX idx_bangumi_episodes_cache_state,
    DROP COLUMN missing_fetch_count,
    DROP COLUMN is_stale,
    DROP COLUMN fetch_generation;
