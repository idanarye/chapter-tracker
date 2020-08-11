CREATE TABLE directories (
	id integer primary key autoincrement,
	series integer,
	pattern text,
	dir text,
	volume integer,
	recursive integer
);
