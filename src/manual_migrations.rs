use sqlx::sqlite::SqlitePool;

pub async fn migrate_manually(pool: &SqlitePool) -> anyhow::Result<()> {
    ensure_table(pool, "media_types", "name text unique, base_dir text, file_types text, program text").await?;
    ensure_table(pool, "serieses", "media_type integer, name text, numbers_repeat_each_volume integer, download_command_dir text, download_command text").await?;
    ensure_table(pool, "episodes", "series integer, number integer, name text, file text, date_of_read datetime, volume integer").await?;
    ensure_table(pool, "directories", "series integer, pattern text, dir text, volume integer, recursive integer").await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS serieses_unique ON serieses(media_type, name)").execute(pool).await?;
    Ok(())
}

async fn ensure_table(pool: &SqlitePool, table_name: &'static str, table_columns: &'static str) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    let orig_sql = if let Some((orig_sql,)) = sqlx::query_as::<_, (String,)>("SELECT sql FROM sqlite_master WHERE type == 'table' AND name = ?").bind(table_name).fetch_optional(&mut tx).await? {
        log::trace!("Original SQL of {} is {:?}. Ensuring correctness.", table_name, orig_sql);
        orig_sql
    } else {
        log::debug!("{} does not exist - creating from scratch.", table_name);
        let statement = format!("CREATE TABLE {} (id integer primary key autoincrement, {})", table_name, table_columns);
        sqlx::query(&statement).execute(&mut tx).await?;
        tx.commit().await?;
        return Ok(());
    };
    let temp_table_name = format!("{}_tmp", table_name);
    let statement = format!("CREATE TABLE {} (id integer primary key autoincrement, {})", temp_table_name, table_columns);
    sqlx::query(&statement).execute(&mut tx).await?;
    let (normalized_sql,) = sqlx::query_as::<_, (String,)>("SELECT sql FROM sqlite_master WHERE type == 'table' AND name = ?").bind(&temp_table_name).fetch_one(&mut tx).await?;
    if orig_sql == normalized_sql.replacen(temp_table_name.as_str(), table_name, 1) {
        log::trace!("SQLs matches - {} is OK", temp_table_name);
        tx.rollback().await?;
        return Ok(());
    }
    todo!("Implement table schema change by copying to the temp table and renaming it to replace the old one");
}
