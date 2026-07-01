RENAME TABLE recording_logs TO recording_logs_legacy;

CREATE TABLE recording_logs (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    recording_id INT UNSIGNED NULL,
    user_id INT UNSIGNED NULL,
    target_type VARCHAR(32) NOT NULL,
    target_id INT UNSIGNED NULL,
    action VARCHAR(64) NOT NULL,
    field_name VARCHAR(64) NULL,
    old_value JSON NULL,
    new_value JSON NULL,
    metadata JSON NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_recording_logs_recording_created (recording_id, created_at),
    INDEX idx_recording_logs_user_created (user_id, created_at),
    INDEX idx_recording_logs_action_created (action, created_at),
    INDEX idx_recording_logs_target (target_type, target_id, created_at)
);

INSERT INTO recording_logs (
    id,
    recording_id,
    user_id,
    target_type,
    target_id,
    action,
    field_name,
    new_value,
    metadata,
    created_at
)
SELECT
    id,
    recording_id,
    user_id,
    'bangumi',
    bangumi_id,
    'recorder_changed',
    'recorder',
    JSON_QUOTE(recorder),
    JSON_OBJECT('migrated_from', 'recording_logs.recorder'),
    created_at
FROM recording_logs_legacy;

DROP TABLE recording_logs_legacy;

CREATE TABLE system_logs (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    level VARCHAR(16) NOT NULL DEFAULT 'info',
    category VARCHAR(64) NOT NULL,
    action VARCHAR(64) NOT NULL,
    message VARCHAR(500) NOT NULL,
    user_id INT UNSIGNED NULL,
    metadata JSON NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_system_logs_category_created (category, created_at),
    INDEX idx_system_logs_level_created (level, created_at),
    INDEX idx_system_logs_user_created (user_id, created_at)
);

CREATE TABLE user_settings (
    user_id INT UNSIGNED NOT NULL,
    setting_key VARCHAR(100) NOT NULL,
    setting_value JSON NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT pk_user_settings PRIMARY KEY (user_id, setting_key),
    CONSTRAINT fk_user_settings_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
