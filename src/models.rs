// use std::time::SystemTime;

#[derive(Debug, Queryable)]
pub struct MediaType {
    pub id: i32,
    pub name: String,
    pub base_dir: String,
    pub file_types: String,
    pub program: String,
}

#[derive(Debug, Queryable)]
pub struct Series {
    pub id: i32,
    pub media_type_id: i32,
    pub name: String,
    pub numbers_repeat_each_volume: Option<bool>,
    pub download_command_dir: Option<String>,
    pub download_command: Option<String>,
}

#[derive(Debug, Queryable)]
pub struct Directory {
    pub id: i32,
    pub series_id: i32,
    pub pattern: String,
    pub dir: String,
    pub volume: Option<i32>,
    pub recursive: Option<bool>,
}

#[derive(Debug, Queryable)]
pub struct Episode {
    pub id: i32,
    pub series_id: i32,
    pub numbers: i32,
    pub name: String,
    pub file: String,
    pub date_of_read: Option<String>,
    pub volume: Option<i32>,
}
