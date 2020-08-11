extern crate chapter_tracker;

extern crate tempfile;

use tempfile::tempdir;

use chapter_tracker::run_migrations;

#[test]
fn test_scan_directory() {
    let dir = tempdir().unwrap();
    let con = chapter_tracker::establish_connection(dir.path().join("db.sqlite").to_str().unwrap());

    run_migrations(&con).unwrap();
}
