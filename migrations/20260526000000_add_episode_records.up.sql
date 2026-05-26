CREATE TABLE episode_records (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    recording_id INT UNSIGNED NOT NULL,
    ordinal INT NOT NULL COMMENT 'Episode number (1-based)',
    watched TINYINT(1) NOT NULL DEFAULT 0,
    progress_seconds INT DEFAULT NULL COMMENT 'Playback position in seconds',
    completed_at DATETIME DEFAULT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE,
    UNIQUE KEY uk_recording_episode (recording_id, ordinal)
);
