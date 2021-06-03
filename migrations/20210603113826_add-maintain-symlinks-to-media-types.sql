-- Add migration script here

ALTER TABLE media_types ADD maintain_symlinks BOOLEAN;
UPDATE media_types SET maintain_symlinks = false WHERE maintain_symlinks IS NULL;
