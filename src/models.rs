#[derive(Debug, sqlx::FromRow)]
pub struct Directory {
    pub id: i64,
    pub series: i64,
    pub pattern: String,
    pub dir: String,
    pub volume: Option<i32>,
    pub recursive: bool,
}
