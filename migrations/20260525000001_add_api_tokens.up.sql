CREATE TABLE IF NOT EXISTS api_tokens (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '' COMMENT 'Token label/name',
    token_hash CHAR(64) NOT NULL COMMENT 'SHA-256 hash of the raw token',
    permissions BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Permission bitmask',
    is_active TINYINT(1) NOT NULL DEFAULT 1,
    last_used_at DATETIME NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY uk_api_tokens_token_hash (token_hash),
    INDEX idx_api_tokens_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Migrate existing tokens from users table to api_tokens
INSERT INTO api_tokens (user_id, name, token_hash, permissions, is_active)
SELECT id, 'Default Token', api_token_hash, ~0, 1
FROM users WHERE api_token_hash != '' AND api_token_hash IS NOT NULL;
