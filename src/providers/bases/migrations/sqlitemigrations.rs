use rusqlite_migration::{M, Migrations};

const MIGRATIONS_COLLECTION: &[M<'_>] = &[M::up(
    "CREATE TABLE Users (Id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), Name TEXT, CreationDate TEXT);",
)];

pub const MIGRATIONS: Migrations = Migrations::from_slice(MIGRATIONS_COLLECTION);
