ALTER TABLE recordings
    MODIFY COLUMN created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    MODIFY COLUMN updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6);

ALTER TABLE episode_records
    MODIFY COLUMN completed_at DATETIME(6) NULL,
    MODIFY COLUMN created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    MODIFY COLUMN updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6);

CREATE TABLE sync_changes (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    bangumi_id VARCHAR(50) NOT NULL,
    entity_type ENUM('record', 'episode') NOT NULL,
    ordinal INT NULL,
    is_delete TINYINT(1) NOT NULL DEFAULT 0,
    changed_at DATETIME(6) NOT NULL,
    INDEX idx_sync_changes_user_cursor (user_id, id),
    INDEX idx_sync_changes_user_subject (user_id, bangumi_id, id),
    CONSTRAINT fk_sync_changes_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO sync_changes (user_id, bangumi_id, entity_type, ordinal, is_delete, changed_at)
SELECT r.user_id, b.external_id, 'record', NULL, r.is_delete, r.updated_at
FROM recordings r
JOIN bangumi_info_easy b ON b.id = r.bangumi_id
WHERE r.bangumi_id IS NOT NULL;

INSERT INTO sync_changes (user_id, bangumi_id, entity_type, ordinal, is_delete, changed_at)
SELECT r.user_id, b.external_id, 'episode', e.ordinal, 0, e.updated_at
FROM episode_records e
JOIN recordings r ON r.id = e.recording_id
JOIN bangumi_info_easy b ON b.id = r.bangumi_id
WHERE r.bangumi_id IS NOT NULL;

CREATE TRIGGER recordings_sync_insert AFTER INSERT ON recordings
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, is_delete, changed_at)
SELECT NEW.user_id, b.external_id, 'record', NEW.is_delete, NEW.updated_at
FROM bangumi_info_easy b WHERE b.id = NEW.bangumi_id;

CREATE TRIGGER recordings_sync_update AFTER UPDATE ON recordings
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, is_delete, changed_at)
SELECT NEW.user_id, b.external_id, 'record', NEW.is_delete, NEW.updated_at
FROM bangumi_info_easy b
WHERE b.id = NEW.bangumi_id
  AND NOT (OLD.recorder <=> NEW.recorder AND OLD.status = NEW.status AND OLD.is_delete = NEW.is_delete);

CREATE TRIGGER recordings_sync_delete BEFORE DELETE ON recordings
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, is_delete, changed_at)
SELECT OLD.user_id, b.external_id, 'record', 1,
       IF(OLD.is_delete = 1, OLD.updated_at, CURRENT_TIMESTAMP(6))
FROM bangumi_info_easy b WHERE b.id = OLD.bangumi_id;

CREATE TRIGGER episode_records_sync_insert AFTER INSERT ON episode_records
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, ordinal, changed_at)
SELECT r.user_id, b.external_id, 'episode', NEW.ordinal, NEW.updated_at
FROM recordings r JOIN bangumi_info_easy b ON b.id = r.bangumi_id
WHERE r.id = NEW.recording_id;

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

CREATE TRIGGER episode_records_sync_delete BEFORE DELETE ON episode_records
FOR EACH ROW
INSERT INTO sync_changes (user_id, bangumi_id, entity_type, ordinal, is_delete, changed_at)
SELECT r.user_id, b.external_id, 'episode', OLD.ordinal, 1, CURRENT_TIMESTAMP(6)
FROM recordings r JOIN bangumi_info_easy b ON b.id = r.bangumi_id
WHERE r.id = OLD.recording_id;
