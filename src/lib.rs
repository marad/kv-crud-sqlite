#[macro_use]
extern crate failure;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod tests;

use kv_crud_core::{Create, Entity, Read, Update};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, Fail)]
pub enum SqliteStorageError {
    #[fail(display = "Value not found for key {}", _0)]
    NotFound(String),

    #[fail(display = "Formatting error: {}", _0)]
    FormattingError(serde_json::Error),

    #[fail(display = "SQLite error: {}", _0)]
    SqliteError(sqlite::Error),

    #[fail(display = "Unknown error")]
    UnknownError,
}

fn wrap_sqlite_error(sqlite_error: sqlite::Error) -> SqliteStorageError {
    SqliteStorageError::SqliteError(sqlite_error)
}

fn wrap_serde_error(serde_error: serde_json::Error) -> SqliteStorageError {
    SqliteStorageError::FormattingError(serde_error)
}

type Result<T> = std::result::Result<T, SqliteStorageError>;

pub struct SqliteStorage {
    connection: sqlite::Connection,
}

impl SqliteStorage {
    pub fn new<T: ToString>(path: T) -> Result<Self> {
        let connection = sqlite::open(path.to_string()).map_err(wrap_sqlite_error)?;

        connection
            .execute("CREATE TABLE IF NOT EXISTS data (key TEXT PRIMARY KEY, value TEXT);")
            .map_err(wrap_sqlite_error)?;

        Ok(Self { connection })
    }
}

impl<I, E> Create<I, E> for SqliteStorage
where
    I: ToString,
    E: Entity<I> + Serialize,
{
    type Error = SqliteStorageError;

    fn save(&mut self, entity: &E) -> Result<()> {
        let mut statement = self
            .connection
            .prepare(
                "INSERT INTO data (key, value) VALUES (?, ?)
                ON CONFLICT (key) DO UPDATE SET value = excluded.value;",
            )
            .map_err(wrap_sqlite_error)?;

        let key: &str = &entity.get_id().to_string();
        let value: &str = &serde_json::to_string(entity).map_err(wrap_serde_error)?;
        statement.bind(1, key).map_err(wrap_sqlite_error)?;
        statement.bind(2, value).map_err(wrap_sqlite_error)?;

        statement.next().map_err(wrap_sqlite_error)?;
        Ok(())
    }
}

impl<I, E> Read<I, E> for SqliteStorage
where
    I: ToString,
    E: Entity<I> + DeserializeOwned,
{
    type Error = SqliteStorageError;

    fn find_by_id(&self, id: &I) -> Result<E> {
        let mut statement = self
            .connection
            .prepare("SELECT value FROM data WHERE key = ?")
            .map_err(wrap_sqlite_error)?;

        let key: &str = &id.to_string();
        statement.bind(1, key).map_err(wrap_sqlite_error)?;

        match statement.next().map_err(wrap_sqlite_error)? {
            sqlite::State::Row => {
                let serialized: String = statement.read(0).map_err(wrap_sqlite_error)?;
                Ok(serde_json::from_str(&serialized).map_err(wrap_serde_error)?)
            }
            sqlite::State::Done => Err(SqliteStorageError::NotFound(key.to_owned())),
        }
    }
}

impl<I, E> Update<I, E> for SqliteStorage
where
    I: ToString,
    E: Entity<I> + Serialize,
{
    type Error = SqliteStorageError;

    fn update(&mut self, entity: &E) -> Result<()> {
        self.save(entity)
    }
}
