use rusqlite_migration::{M, Migrations};

const MIGRATIONS_COLLECTION: &[M<'_>] = &[
    M::up(
        "CREATE TABLE IF NOT EXISTS Users (Id INTEGER PRIMARY KEY, Name TEXT, CreationDate DATE);",
    ),
    M::up(
        "CREATE TABLE IF NOT EXISTS NewUsers (Id INTEGER PRIMARY KEY, Name TEXT, Number Text UNIQUE, CreationDate DATE);",
    ),
    M::up(
        "INSERT INTO NewUsers(Id, Name, Number,CreationDate) Select Id, Name, lower(hex(randomblob(6))), CreationDate from Users;",
    ),
    M::up("DROP TABLE Users;"),
    M::up("ALTER TABLE NewUsers rename to Users"),
    M::up(
        "CREATE TABLE IF NOT EXISTS Accounts (Id INTEGER PRIMARY KEY, Name Text, UserId INTEGER, Count Decimal , FOREIGN KEY(UserId) REFERENCES Users(Id));",
    ),
    M::up("PRAGMA foreign_keys=ON;"),
    M::up("ALTER TABLE Accounts rename Count to MoneyCount; "),
    M::up("ALTER TABLE Accounts Add CreationDate; "),
];

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
                println!("{}", er);
                assert!(false);
            }
        }
    }
}
