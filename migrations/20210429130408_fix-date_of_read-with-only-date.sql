-- Add migration script here
UPDATE episodes SET date_of_read = date_of_read || ' 00:00:00' WHERE date_of_read LIKE '____-__-__';
