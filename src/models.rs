#[derive(PartialEq, Debug, sqlx::FromRow)]
pub struct MediaType {
    pub id: i64,
    pub name: String,
    pub base_dir: String,
    pub file_types: String,
    pub adjacent_file_types: String,
    pub program: String,
    pub maintain_symlinks: bool,
}

#[derive(PartialEq, Debug, Clone, sqlx::FromRow)]
pub struct Series {
    pub id: i64,
    pub media_type: i64,
    pub name: String,
    // pub numbers_repeat_each_volume: Option<bool>,
    pub download_command_dir: Option<String>,
    pub download_command: Option<String>,
}

#[derive(PartialEq, Debug, sqlx::FromRow)]
pub struct SeriesReadStats {
    pub num_episodes: i32,
    pub num_unread: i32,
}

#[derive(PartialEq, Debug, sqlx::FromRow)]
pub struct Directory {
    pub id: i64,
    pub series: i64,
    pub pattern: String,
    pub dir: String,
    pub volume: Option<i32>,
    pub recursive: bool,
}

#[derive(PartialEq, Debug, sqlx::FromRow)]
pub struct Episode {
    pub id: i64,
    pub series: i64,
    pub number: i64,
    pub name: String,
    pub file: String,
    pub volume: Option<i64>,
    pub date_of_read: Option<sqlx::types::chrono::NaiveDateTime>,
}
