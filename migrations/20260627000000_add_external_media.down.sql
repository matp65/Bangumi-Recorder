ALTER TABLE recordings DROP FOREIGN KEY fk_recordings_external_media_id;

ALTER TABLE recordings DROP INDEX uk_recordings_user_external_media;

ALTER TABLE recordings DROP COLUMN external_media_id;

DROP TABLE IF EXISTS external_media_detailed;

DROP TABLE IF EXISTS external_media;

ALTER TABLE bangumi_info_easy
    MODIFY COLUMN type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other';

ALTER TABLE bangumi_info_detailed
    MODIFY COLUMN type TINYINT NOT NULL COMMENT '1=TV, 2=Movie, 3=OVA, 4=ONA, 5=TV Short, 6=Music, 7=Book, 8=Other';
