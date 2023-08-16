-- Add migration script here

ALTER TABLE media_types ADD adjacent_file_types TEXT;
UPDATE media_types SET adjacent_file_types = '' WHERE adjacent_file_types IS NULL;
