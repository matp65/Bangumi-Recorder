ALTER TABLE bangumi_episodes
    ADD COLUMN ep_label VARCHAR(20) DEFAULT NULL COMMENT 'Display label (e.g. "1", "SP1")' AFTER ordinal;
