ALTER TABLE recordings ADD CONSTRAINT uk_recordings_user_bangumi UNIQUE (user_id, bangumi_id);
