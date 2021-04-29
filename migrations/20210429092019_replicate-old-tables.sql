-- Add migration script here

CREATE TABLE IF NOT EXISTS media_types (id integer primary key autoincrement, name text unique, base_dir text, file_types text, program text);
CREATE TABLE IF NOT EXISTS serieses (id integer primary key autoincrement, media_type integer, name text, numbers_repeat_each_volume integer, download_command_dir text, download_command text);
CREATE UNIQUE INDEX IF NOT EXISTS serieses_unique ON serieses(media_type,name);
CREATE TABLE IF NOT EXISTS directories (id integer primary key autoincrement, series integer, pattern text, dir text, volume integer, recursive integer);
CREATE TABLE IF NOT EXISTS episodes (id integer primary key autoincrement, series integer, number integer, name text, file text, date_of_read datetime, volume integer);
