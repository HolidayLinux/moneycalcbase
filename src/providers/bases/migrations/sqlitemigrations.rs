use rusqlite_migration::{M, Migrations};

const MIGRATIONS_COLLECTION: &[M<'_>] = &[M::up(
    "CREATE TABLE Users (Id INTEGER PRIMARY KEY, Name TEXT, CreationDate DATE);",
)];

pub const MIGRATIONS: Migrations = Migrations::from_slice(MIGRATIONS_COLLECTION);
