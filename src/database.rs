pub struct Database {
    connection: sqlx::SqlitePool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let connection_url = if url.starts_with("sqlite:") {
            url.to_owned()
        } else {
            format!("{url}")
        };

        let connection = sqlx::SqlitePool::connect(&connection_url).await?;
        Ok(Self { connection })
    }

    pub fn get_connection(&self) -> &sqlx::SqlitePool {
        &self.connection
    }
}
