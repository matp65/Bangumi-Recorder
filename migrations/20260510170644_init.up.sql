-- Add up migration script here
CREATE TABLE users (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    api_token_hash CHAR(64) NOT NULL,
    status TINYINT NOT NULL DEFAULT 1 COMMENT '1=active, 0=disabled',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT uk_users_username UNIQUE (username),
    CONSTRAINT uk_users_api_token UNIQUE (api_token_hash)
);

CREATE TABLE bangumi_info_easy (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    external_id VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other',
    info TEXT,
    cover_url VARCHAR(255),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT uk_bangumi_info_external_id UNIQUE (external_id)
);

CREATE TABLE bangumi_info_detailed (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    bangumi_id INT UNSIGNED NOT NULL,
    type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other',
    author VARCHAR(255),
    release_date DATE,
    episodes INT,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_bangumi_info_detailed_bangumi_id FOREIGN KEY (bangumi_id) REFERENCES bangumi_info_easy(id) ON DELETE CASCADE
);

CREATE TABLE recordings (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    bangumi_id INT UNSIGNED NOT NULL,
    status TINYINT NOT NULL DEFAULT 0 COMMENT '0=pending, 1=recording, 2=completed, 3=failed',
    recorder VARCHAR(255),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_recordings_bangumi_id FOREIGN KEY (bangumi_id) REFERENCES bangumi_info_easy(id) ON DELETE CASCADE,
    CONSTRAINT fk_recordings_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);