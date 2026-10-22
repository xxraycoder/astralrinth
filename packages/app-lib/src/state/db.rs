use crate::ErrorKind;
use crate::state::DirectoryInfo;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::time::Duration;

pub(crate) async fn connect(
    app_identifier: &str,
) -> crate::Result<Pool<Sqlite>> {
    let settings_dir = DirectoryInfo::initial_settings_dir_path(app_identifier)
        .ok_or(ErrorKind::FSError(
            "could not find valid config dir".to_string(),
        ))?;

    crate::util::io::create_dir_all(&settings_dir).await?;

    let db_path = settings_dir.join("app.db");

    connect_app_db(&db_path).await
}

async fn connect_app_db(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
	super::db_backup::maybe_backup_existing_app_db(db_path).await?;
    open_migrated_app_db(db_path).await
}

async fn open_migrated_app_db(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
    let pool = open_app_db_pool(db_path).await?;

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database before migrations: {err}"
        );
    }

    sqlx::migrate!().run(&pool).await?;
    record_current_app_version(&pool).await?;

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database: {err}"
        );
    }

    Ok(pool)
}

async fn open_app_db_pool(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
    let conn_options = SqliteConnectOptions::new()
        .filename(db_path)
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .optimize_on_close(true, None)
        .create_if_missing(true);

    Ok(SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .idle_timeout(Duration::from_secs(120))
        .connect_with(conn_options)
        .await?)
}

async fn record_current_app_version(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let previous_version = sqlx::query_scalar!(
        "SELECT value FROM app_metadata WHERE key = 'app_version'"
    )
    .fetch_optional(pool)
    .await?;
    let already_used_sync_update = previous_version
        .as_deref()
        .and_then(|version| {
            let mut parts = version.split('.');
            Some((
                parts.next()?.parse::<u64>().ok()?,
                parts.next()?.parse::<u64>().ok()?,
            ))
        })
        .is_some_and(|version| version >= (0, 20));

    if env!("CARGO_PKG_VERSION").starts_with("0.20.")
        && already_used_sync_update
    {
        let mut settings = super::Settings::get(pool).await?;
        if settings.pending_update_toast_for_version.is_some() {
            settings.pending_update_toast_for_version = None;
            settings.update(pool).await?;
        }
    }

    sqlx::query!(
        "
		INSERT INTO app_metadata (key, value, updated_at)
		VALUES ('app_version', ?, unixepoch())
		ON CONFLICT(key) DO UPDATE SET
			value = excluded.value,
			updated_at = excluded.updated_at
		",
        env!("CARGO_PKG_VERSION"),
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Cleans up data from the database that is no longer referenced, but must be
/// kept around for a little while to allow users to recover from accidental
/// deletions.
async fn stale_data_cleanup(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let has_skin_tables = sqlx::query!(
		"SELECT COUNT(*) AS \"count!: i64\" FROM sqlite_master WHERE type = 'table' AND name IN ('custom_minecraft_skins', 'minecraft_users')",
	)
	.fetch_one(&mut *tx)
	.await?
	.count == 2;

    if has_skin_tables {
        sqlx::query!(
			"DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
		)
		.execute(&mut *tx)
		.await?;
    }

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
	use super::connect_app_db;
	use sqlx::sqlite::SqliteConnectOptions;
	use sqlx::{ConnectOptions, Connection};

	#[tokio::test]
	async fn astralrinth_backup_failure_prevents_migrations() {
		let directory = tempfile::tempdir().unwrap();
		let db_path = directory.path().join("app.db");
		let mut connection = SqliteConnectOptions::new()
			.filename(&db_path)
			.create_if_missing(true)
			.connect()
			.await
			.unwrap();
		sqlx::query("CREATE TABLE preserved_data (value TEXT NOT NULL)")
			.execute(&mut connection)
			.await
			.unwrap();
		sqlx::query("INSERT INTO preserved_data VALUES ('keep me')")
			.execute(&mut connection)
			.await
			.unwrap();
		sqlx::query("CREATE TABLE app_metadata (key TEXT PRIMARY KEY)")
			.execute(&mut connection)
			.await
			.unwrap();
		connection.close().await.unwrap();
		let before = tokio::fs::read(&db_path).await.unwrap();

		let error = connect_app_db(&db_path).await.unwrap_err();

		assert!(matches!(
			error.downcast_ref::<sqlx::Error>(),
			Some(sqlx::Error::Database(_))
		));
		assert!(error.to_string().contains("no such column: value"));
		assert_eq!(tokio::fs::read(&db_path).await.unwrap(), before);
		let mut connection = SqliteConnectOptions::new()
			.filename(&db_path)
			.read_only(true)
			.connect()
			.await
			.unwrap();
		let tables = sqlx::query_scalar::<_, String>(
			"SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
		)
		.fetch_all(&mut connection)
		.await
		.unwrap();
		assert_eq!(tables, vec!["app_metadata", "preserved_data"]);
		let value = sqlx::query_scalar::<_, String>(
			"SELECT value FROM preserved_data",
		)
		.fetch_one(&mut connection)
		.await
		.unwrap();
		assert_eq!(value, "keep me");
		connection.close().await.unwrap();
	}
}
