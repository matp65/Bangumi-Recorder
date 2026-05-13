CREATE TABLE recording_logs (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    recording_id INT UNSIGNED NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    bangumi_id INT UNSIGNED NOT NULL,
    recorder VARCHAR(255) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_recording_logs_recording_id FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE,
    CONSTRAINT fk_recording_logs_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_recording_logs_bangumi_id FOREIGN KEY (bangumi_id) REFERENCES bangumi_info_easy(id) ON DELETE CASCADE
);
