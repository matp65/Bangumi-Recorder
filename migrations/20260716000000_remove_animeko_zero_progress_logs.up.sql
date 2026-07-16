-- Animeko may serialize an existing zero-position playback history as 0 while
-- Bangumi Recorder stores an untouched position as NULL. The sync write keeps
-- both snapshots aligned, but the representation-only change is not useful in
-- the user-facing recording history.
DELETE FROM recording_logs
WHERE action = 'episode_updated'
  AND JSON_UNQUOTE(JSON_EXTRACT(metadata, '$.source')) = 'animeko'
  AND (
      (
          JSON_TYPE(JSON_EXTRACT(old_value, '$.progress_seconds')) = 'NULL'
          AND JSON_EXTRACT(new_value, '$.progress_seconds') = 0
      )
      OR (
          JSON_EXTRACT(old_value, '$.progress_seconds') = 0
          AND JSON_TYPE(JSON_EXTRACT(new_value, '$.progress_seconds')) = 'NULL'
      )
  )
  AND JSON_EXTRACT(old_value, '$.ordinal') <=> JSON_EXTRACT(new_value, '$.ordinal')
  AND JSON_EXTRACT(old_value, '$.watched') <=> JSON_EXTRACT(new_value, '$.watched')
  AND JSON_EXTRACT(old_value, '$.duration_seconds') <=> JSON_EXTRACT(new_value, '$.duration_seconds')
  AND JSON_EXTRACT(old_value, '$.completed_at') <=> JSON_EXTRACT(new_value, '$.completed_at');
