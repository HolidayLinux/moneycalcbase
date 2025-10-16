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
        let connect: Connection;
        if config.memory_base {
            connect = Connection::open_in_memory()?;
        } else {
            connect = Connection::open(config.connection_string.as_str())?;
        }

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
            "insert into Users(Name, Number, CreationDate) values (?1,?2, ?3);",
            [
                user.name.as_str(),
                user.number.as_str(),
                user.creation_date.to_string().as_str(),
            ],
        ) {
            Ok(res) => return Ok(()),
            Err(e) => return Err(e),
        }
    }

    fn get_users(&self) -> Result<Vec<User>, Error> {
        let mut values = self.connection.prepare("select * from Users;")?;
        let rows = values.query_map([], |row| {
            Ok(User::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;

        let mut users: Vec<User> = vec![];
        rows.into_iter().for_each(|user| match user {
            Ok(res) => users.push(res),
            Err(_) => (),
        });
        Ok(users)
    }

    fn get_user_by_number(&self, number: &str) -> Result<User, Error> {
        self.connection
            .query_one("Select * from users where Number = ?1", [number], |row| {
                let user = User::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
                Ok(user)
            })
    }

    fn delete_user_by_id(&self, id: i32) -> Result<(), Error> {
        match self
            .connection
            .execute("Delete from Users where Id = ?1", [id])
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
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
    async fn create_test_file_base() {
        let config = Configuration::new("./testbases/testbase.db3");
        let _ = SqliteProvider::new(&config, false);
        assert!(check_exist(config.connection_string.as_str()).await);
    }

    #[test]
    fn create_test_memory_base() {
        let config = Configuration::memory_base();
        let _ = SqliteProvider::new(&config, false).unwrap();
    }

    #[tokio::test]
    async fn add_migration_test() {
        let config = Configuration::new("./testbases/testbase_mig.db3");
        let _ = SqliteProvider::new(&config, true);
        assert!(check_exist(config.connection_string.as_str()).await);
    }

    #[tokio::test]
    async fn add_user_test() {
        let config = Configuration::memory_base();
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let user = User::new(
            1,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        sqlite_provider.add_user(&user).unwrap();

        sqlite_provider.delete_user_by_id(user.id).unwrap();
    }

    #[tokio::test]
    async fn add_user_notunique_test() {
        let config = Configuration::new("./testbases/testbase_user.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let user = User::new(
            1,
            String::from_str("scam").unwrap(),
            String::from_str("88005553535").unwrap(),
            String::from_str("2025-10-10").unwrap(),
        );

        assert!(sqlite_provider.add_user(&user).is_err());
    }

    #[tokio::test]
    async fn get_users_test() {
        let config = Configuration::new("./testbases/testbase_users.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();

        let user = User::new(
            1,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        sqlite_provider.add_user(&user).unwrap();
        let user = User::new(
            2,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        sqlite_provider.add_user(&user).unwrap();

        let users = sqlite_provider.get_users().unwrap();
        assert!(users.len() > 0);

        fs::remove_file("./testbases/testbase_users.db3")
            .await
            .unwrap();
    }

    #[test]
    fn get_user_by_number_test() {
        let config = Configuration::new("./testbases/testbase_user.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let user_res = sqlite_provider.get_user_by_number("88005553535");
        match user_res {
            Ok(user) => {
                assert!(user.name.eq("scam"));
            }
            Err(err) => {
                panic!("{}", err)
            }
        }
    }

    #[test]
    fn delete_users_test() {
        let config = Configuration::new("./testbases/testbase_user_delete.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();

        let user = User::new(
            1,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        sqlite_provider.add_user(&user).unwrap();
        let user = User::new(
            2,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        sqlite_provider.add_user(&user).unwrap();

        let users = sqlite_provider.get_users().unwrap();
        assert!(users.len() > 0);

        users.iter().for_each(|user| {
            sqlite_provider.delete_user_by_id(user.id).unwrap();
        });

        let users = sqlite_provider.get_users().unwrap();

        assert!(users.len() == 0);

        std::fs::remove_file("./testbases/testbase_user_delete.db3").unwrap();
    }

    async fn check_exist(path: &str) -> bool {
        let meta = fs::metadata(path).await.ok();
        if let Some(_) = meta { true } else { false }
    }
}
