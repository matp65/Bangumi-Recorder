DROP TABLE IF EXISTS user_settings;
DROP TABLE IF EXISTS system_logs;

CREATE TABLE recording_logs_legacy (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    recording_id INT UNSIGNED NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    bangumi_id INT UNSIGNED NOT NULL,
    recorder VARCHAR(255) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO recording_logs_legacy (recording_id, user_id, bangumi_id, recorder, created_at)
SELECT recording_id, user_id, target_id, JSON_UNQUOTE(new_value), created_at
FROM recording_logs
WHERE target_type = 'bangumi'
  AND action = 'recorder_changed'
  AND field_name = 'recorder'
  AND recording_id IS NOT NULL
  AND target_id IS NOT NULL
  AND user_id IS NOT NULL
  AND new_value IS NOT NULL;

DROP TABLE recording_logs;
RENAME TABLE recording_logs_legacy TO recording_logs;
