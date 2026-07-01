CREATE TABLE external_media (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    source VARCHAR(20) NOT NULL COMMENT 'External source, e.g. imdb',
    external_id VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other, 9=Game, 10=Real',
    info TEXT,
    cover_url TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT uk_external_media_source_external UNIQUE (source, external_id),
    INDEX idx_external_media_title (title)
);

CREATE TABLE external_media_detailed (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    media_id INT UNSIGNED NOT NULL,
    author VARCHAR(255),
    release_date DATE,
    episodes INT,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_external_media_detailed_media_id FOREIGN KEY (media_id) REFERENCES external_media(id) ON DELETE CASCADE,
    CONSTRAINT uk_external_media_detailed_media UNIQUE (media_id)
);

ALTER TABLE recordings ADD COLUMN external_media_id INT UNSIGNED NULL AFTER other_id;

ALTER TABLE recordings
    ADD CONSTRAINT fk_recordings_external_media_id FOREIGN KEY (external_media_id) REFERENCES external_media(id) ON DELETE SET NULL;

ALTER TABLE recordings
    ADD CONSTRAINT uk_recordings_user_external_media UNIQUE (user_id, external_media_id);

ALTER TABLE bangumi_info_easy
    MODIFY COLUMN type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other, 9=Game, 10=Real';

ALTER TABLE bangumi_info_detailed
    MODIFY COLUMN type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other, 9=Game, 10=Real';
