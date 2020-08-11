CREATE TABLE episodes (
	id integer primary key autoincrement,
	series integer,
	number integer,
	name text,
	file text,
	date_of_read datetime,
	volume integer
);
