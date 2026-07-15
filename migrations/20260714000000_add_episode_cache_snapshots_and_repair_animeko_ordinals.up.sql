ALTER TABLE bangumi_episodes
    ADD COLUMN fetch_generation CHAR(36) NULL
        COMMENT 'Identifier of the last complete fetch that observed this episode' AFTER duration,
    ADD COLUMN is_stale TINYINT(1) NOT NULL DEFAULT 0
        COMMENT '1 when the latest complete fetch did not contain this episode' AFTER fetch_generation,
    ADD COLUMN missing_fetch_count TINYINT UNSIGNED NOT NULL DEFAULT 0
        COMMENT 'Consecutive complete fetches that did not contain this episode' AFTER is_stale,
    ADD INDEX idx_bangumi_episodes_cache_state (bangumi_easy_id, is_stale, updated_at);

-- Ordinal-only updates must be visible to Animeko's cursor stream as well.
DROP TRIGGER IF EXISTS episode_records_sync_update;
CREATE TRIGGER episode_records_sync_update AFTER UPDATE ON episode_records
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, ordinal, changed_at)
SELECT r.user_id, b.external_id, 'episode', NEW.ordinal, NEW.updated_at
FROM recordings r JOIN bangumi_info_easy b ON b.id = r.bangumi_id
WHERE r.id = NEW.recording_id
  AND NOT (
      OLD.ordinal = NEW.ordinal
      AND OLD.watched = NEW.watched
      AND OLD.progress_seconds <=> NEW.progress_seconds
      AND OLD.duration_seconds <=> NEW.duration_seconds
      AND OLD.completed_at <=> NEW.completed_at
  );

-- Animeko historically sent season-relative ordinals (1, 2, ...) for subjects whose
-- Bangumi episode list starts at a global ordinal such as 74. Only repair subjects
-- with a clear offset (the first ordinal is larger than the cached regular-episode
-- count). If a target record already exists, the newer client timestamp wins.
DROP TABLE IF EXISTS _br_migration_20260714_episode_ordinal_repairs;
CREATE TABLE _br_migration_20260714_episode_ordinal_repairs (
    episode_record_id INT UNSIGNED NOT NULL PRIMARY KEY,
    target_episode_record_id INT UNSIGNED NULL,
    old_ordinal INT NOT NULL,
    new_ordinal INT NOT NULL
) ENGINE=InnoDB;

INSERT INTO _br_migration_20260714_episode_ordinal_repairs (
    episode_record_id,
    target_episode_record_id,
    old_ordinal,
    new_ordinal
)
SELECT er.id, target_record.id, er.ordinal, ranked.ordinal
FROM episode_records er
JOIN recordings r ON r.id = er.recording_id
JOIN (
    SELECT
        be.bangumi_easy_id,
        be.ordinal,
        ROW_NUMBER() OVER (
            PARTITION BY be.bangumi_easy_id
            ORDER BY be.ordinal ASC
        ) AS subject_position
    FROM bangumi_episodes be
    JOIN (
        SELECT bangumi_easy_id
        FROM bangumi_episodes
        WHERE ordinal > 0
        GROUP BY bangumi_easy_id
        HAVING MIN(ordinal) > COUNT(*)
    ) offset_subject ON offset_subject.bangumi_easy_id = be.bangumi_easy_id
    WHERE be.ordinal > 0
) ranked
    ON ranked.bangumi_easy_id = r.bangumi_id
   AND ranked.subject_position = er.ordinal
LEFT JOIN bangumi_episodes exact_episode
    ON exact_episode.bangumi_easy_id = r.bangumi_id
   AND exact_episode.ordinal = er.ordinal
LEFT JOIN episode_records target_record
    ON target_record.recording_id = er.recording_id
   AND target_record.ordinal = ranked.ordinal
WHERE er.ordinal > 0
  AND exact_episode.id IS NULL;

INSERT INTO recording_logs (
    recording_id,
    user_id,
    target_type,
    target_id,
    action,
    field_name,
    old_value,
    new_value,
    metadata
)
SELECT
    er.recording_id,
    r.user_id,
    'bangumi',
    r.bangumi_id,
    'episode_updated',
    'ordinal',
    JSON_OBJECT('ordinal', repair.old_ordinal),
    JSON_OBJECT('ordinal', repair.new_ordinal),
    JSON_OBJECT(
        'source', 'migration',
        'reason', 'repair_animeko_season_relative_ordinal',
        'ordinal', repair.new_ordinal,
        'old_ordinal', repair.old_ordinal
    )
FROM _br_migration_20260714_episode_ordinal_repairs repair
JOIN episode_records er ON er.id = repair.episode_record_id
JOIN recordings r ON r.id = er.recording_id;

INSERT INTO system_logs (level, category, action, message, metadata)
SELECT
    'info',
    'episode_sync',
    'animeko_episode_ordinals_repaired',
    CONCAT(
        'Repaired ', COUNT(*), ' Animeko episode ordinal(s) for Bangumi Subject ', b.external_id
    ),
    JSON_OBJECT(
        'subject_id', b.external_id,
        'repair_count', COUNT(*),
        'ordinals', JSON_ARRAYAGG(
            JSON_OBJECT('old', repair.old_ordinal, 'new', repair.new_ordinal)
        )
    )
FROM _br_migration_20260714_episode_ordinal_repairs repair
JOIN episode_records er ON er.id = repair.episode_record_id
JOIN recordings r ON r.id = er.recording_id
JOIN bangumi_info_easy b ON b.id = r.bangumi_id
GROUP BY b.id, b.external_id;

UPDATE episode_records er
JOIN _br_migration_20260714_episode_ordinal_repairs repair ON repair.episode_record_id = er.id
SET er.ordinal = repair.new_ordinal,
    er.updated_at = CURRENT_TIMESTAMP(6)
WHERE repair.target_episode_record_id IS NULL;

-- If the corrected ordinal already has a record, keep the state with the newer
-- client timestamp and remove only the invalid season-relative source row.
UPDATE episode_records target
JOIN _br_migration_20260714_episode_ordinal_repairs repair
    ON repair.target_episode_record_id = target.id
JOIN episode_records source
    ON source.id = repair.episode_record_id
SET target.watched = IF(source.updated_at > target.updated_at, source.watched, target.watched),
    target.progress_seconds = IF(
        source.updated_at > target.updated_at,
        source.progress_seconds,
        target.progress_seconds
    ),
    target.duration_seconds = IF(
        source.updated_at > target.updated_at,
        source.duration_seconds,
        target.duration_seconds
    ),
    target.completed_at = IF(
        source.updated_at > target.updated_at,
        source.completed_at,
        target.completed_at
    ),
    target.updated_at = CURRENT_TIMESTAMP(6);

DELETE source
FROM episode_records source
JOIN _br_migration_20260714_episode_ordinal_repairs repair ON repair.episode_record_id = source.id
WHERE repair.target_episode_record_id IS NOT NULL;

DROP TABLE _br_migration_20260714_episode_ordinal_repairs;
