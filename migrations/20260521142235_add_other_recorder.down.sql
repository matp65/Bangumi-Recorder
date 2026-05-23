ALTER TABLE recordings DROP FOREIGN KEY fk_recordings_other_id;

ALTER TABLE recordings DROP INDEX uk_recordings_user_other;

ALTER TABLE other_recorders DROP FOREIGN KEY fk_recordings_add_user;

ALTER TABLE other_recorders DROP FOREIGN KEY fk_recordings_update_user;

ALTER TABLE recordings DROP COLUMN other_id;

ALTER TABLE recordings MODIFY COLUMN bangumi_id INT UNSIGNED NOT NULL;

DROP TABLE IF EXISTS other_recorders;