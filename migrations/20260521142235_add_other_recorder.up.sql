ALTER TABLE recordings ADD COLUMN other_id INT UNSIGNED NULL AFTER bangumi_id;
ALTER TABLE recordings DROP FOREIGN KEY fk_recordings_bangumi_id;
ALTER TABLE recordings MODIFY COLUMN bangumi_id INT UNSIGNED NULL;
ALTER TABLE recordings ADD CONSTRAINT fk_recordings_bangumi_id FOREIGN KEY (bangumi_id) REFERENCES bangumi_info_easy(id) ON DELETE CASCADE;

CREATE TABLE other_recorders (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NULL,
    description TEXT NULL,
    cover_url VARCHAR(255) NULL,
    max_number INT NULL,
    status TINYINT NULL,
    add_user INT UNSIGNED NULL,
    update_user INT UNSIGNED NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_recordings_add_user FOREIGN KEY (add_user) REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT fk_recordings_update_user FOREIGN KEY (update_user) REFERENCES users(id) ON DELETE SET NULL
);

ALTER TABLE recordings ADD CONSTRAINT fk_recordings_other_id FOREIGN KEY (other_id) REFERENCES other_recorders(id) ON DELETE CASCADE;

ALTER TABLE recordings ADD CONSTRAINT uk_recordings_user_other UNIQUE (user_id, other_id);