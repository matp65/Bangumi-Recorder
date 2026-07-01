ALTER TABLE users ADD COLUMN is_admin TINYINT NOT NULL DEFAULT 0 AFTER status;

UPDATE users
SET is_admin = 1
WHERE id = (
    SELECT id FROM (
        SELECT id FROM users ORDER BY id LIMIT 1
    ) AS first_user
);
