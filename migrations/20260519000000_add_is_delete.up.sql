ALTER TABLE recordings ADD COLUMN is_delete TINYINT NOT NULL DEFAULT 0 COMMENT '0=active, 1=soft-deleted' AFTER recorder;

UPDATE recordings SET is_delete = 0 WHERE is_delete IS NULL;
