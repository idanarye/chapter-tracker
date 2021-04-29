-- Add migration script here

UPDATE directories SET volume = NULL WHERE volume == '';
UPDATE episodes SET volume = NULL WHERE volume == '';
UPDATE episodes SET volume = cast(volume AS int) WHERE typeof(volume) == 'real';
