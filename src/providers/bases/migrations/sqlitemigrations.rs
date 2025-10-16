use rusqlite_migration::{M, Migrations};

const MIGRATIONS_COLLECTION: &[M<'_>] = &[M::up(
    "CREATE TABLE IF NOT EXISTS Users (Id INTEGER PRIMARY KEY, Name TEXT, CreationDate DATE);",
)];

pub const MIGRATIONS: Migrations = Migrations::from_slice(MIGRATIONS_COLLECTION);

#[cfg(test)]
mod tests {
    use crate::providers::bases::migrations::sqlitemigrations::MIGRATIONS;

    #[test]
    pub fn migrations_validate_test() {
        let validation = MIGRATIONS.validate();
        match validation {
            Ok(_) => assert!(true),
            Err(er) => {
                println!("{:?}", er);
                assert!(false);
            }
        }
    }
}
