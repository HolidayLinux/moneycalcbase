use std::str::FromStr;

use crate::{
    config::Configuration,
    models::user::User,
    providers::{UserProvider, bases::migrations::sqlitemigrations::MIGRATIONS},
};
use rusqlite::{Connection, Error};

#[derive(Debug)]
pub struct SqliteProvider {
    connection: Connection,
}

impl SqliteProvider {
    pub fn new(config: &Configuration, apply_migrations: bool) -> Result<Self, Error> {
        let connect = Connection::open(config.connection_string.as_str())?;

        if apply_migrations {
            let mut mutcon = connect;

            MIGRATIONS.to_latest(&mut mutcon).unwrap();

            Ok(Self { connection: mutcon })
        } else {
            Ok(Self {
                connection: connect,
            })
        }
    }
}

impl UserProvider for SqliteProvider {
    fn add_user(&self, user: &User) -> Result<(), Error> {
        match self.connection.execute(
            "insert into Users(Name, CreationDate) values (?1, ?2);",
            [user.name.as_str(), user.creation_date.to_string().as_str()],
        ) {
            Ok(res) => return Ok(()),
            Err(e) => return Err(e),
        }
    }

    fn get_users(&self) -> Result<Vec<User>, Error> {
        let mut values = self.connection.prepare("select * from Users;")?;
        let rows = values.query_map([], |row| {
            Ok(User::new(row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let mut users: Vec<User> = vec![];
        rows.into_iter().for_each(|user| match user {
            Ok(res) => users.push(res),
            Err(_) => (),
        });
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tokio::fs;

    use crate::{
        config::Configuration,
        models::user::User,
        providers::{UserProvider, bases::sqlite::SqliteProvider},
    };

    #[tokio::test]
    async fn create_test_base() {
        let config = Configuration {
            connection_string: "./testbases/testbase.db3".to_owned(),
        };
        let _ = SqliteProvider::new(&config, false);
        assert!(check_exist(config.connection_string.as_str()).await);
    }

    #[tokio::test]
    async fn add_migration_test() {
        let config = Configuration {
            connection_string: "./testbases/testbase_mig.db3".to_owned(),
        };
        let _ = SqliteProvider::new(&config, true);
        assert!(check_exist(config.connection_string.as_str()).await);
    }

    #[tokio::test]
    async fn add_user_test() {
        let config = Configuration {
            connection_string: "./testbases/testbase_user.db3".to_owned(),
        };
        let sqlite_provider = SqliteProvider::new(&config, false).unwrap();
        let user = User::new(
            String::from_str("1").unwrap(),
            String::from_str("peter").unwrap(),
            String::from_str("2025-09-30").unwrap(),
        );
        sqlite_provider.add_user(&user).unwrap();
    }

    #[test]
    fn get_users_test() {
        let config = Configuration {
            connection_string: "./testbases/testbase_user.db3".to_owned(),
        };
        let sqlite_provider = SqliteProvider::new(&config, false).unwrap();
        let users = sqlite_provider.get_users().unwrap();
        assert_eq!(users.len() > 0, true);
    }

    async fn check_exist(path: &str) -> bool {
        let meta = fs::metadata(path).await.ok();
        if let Some(_) = meta { true } else { false }
    }
}
