CREATE TABLE media_types (
	id integer primary key autoincrement,
	name text unique,
	base_dir text,
	file_types text,
	program text
);
