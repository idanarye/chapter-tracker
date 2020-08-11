CREATE TABLE serieses (
	id integer primary key autoincrement,
	media_type integer,
	name text,
	numbers_repeat_each_volume integer,
	download_command_dir text,
	download_command text
);
