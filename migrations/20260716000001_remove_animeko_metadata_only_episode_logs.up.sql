-- The recording-log list summarizes episode changes by ordinal, watched state,
-- playback position, and duration. Animeko completion-time backfills and the
-- NULL/0 representation of an empty position should not appear as changes when
-- those user-visible values are unchanged.
DELETE FROM recording_logs
WHERE action = 'episode_updated'
  AND JSON_UNQUOTE(JSON_EXTRACT(metadata, '$.source')) = 'animeko'
  AND JSON_EXTRACT(old_value, '$.ordinal') <=> JSON_EXTRACT(new_value, '$.ordinal')
  AND JSON_EXTRACT(old_value, '$.watched') <=> JSON_EXTRACT(new_value, '$.watched')
  AND JSON_EXTRACT(old_value, '$.duration_seconds') <=> JSON_EXTRACT(new_value, '$.duration_seconds')
  AND (
      JSON_EXTRACT(old_value, '$.progress_seconds')
          <=> JSON_EXTRACT(new_value, '$.progress_seconds')
      OR (
          JSON_TYPE(JSON_EXTRACT(old_value, '$.progress_seconds')) = 'NULL'
          AND JSON_EXTRACT(new_value, '$.progress_seconds') = 0
      )
      OR (
          JSON_EXTRACT(old_value, '$.progress_seconds') = 0
          AND JSON_TYPE(JSON_EXTRACT(new_value, '$.progress_seconds')) = 'NULL'
      )
  );
