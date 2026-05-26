CREATE TABLE bangumi_episodes (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    bangumi_easy_id INT UNSIGNED NOT NULL,
    ordinal INT NOT NULL COMMENT 'Episode number (1-based)',
    title VARCHAR(255) DEFAULT NULL,
    name_cn VARCHAR(255) DEFAULT NULL,
    airdate DATE DEFAULT NULL,
    duration VARCHAR(50) DEFAULT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (bangumi_easy_id) REFERENCES bangumi_info_easy(id) ON DELETE CASCADE,
    UNIQUE KEY uk_bangumi_episode (bangumi_easy_id, ordinal)
);
