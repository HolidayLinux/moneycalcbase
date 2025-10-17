use crate::{
    config::Configuration,
    models::{
        account::Account,
        moneytransaction::{MoneyTransaction, PaymentType},
        user::User,
    },
    providers::{
        AccountProvider, TransactionWorker, UserProvider,
        bases::migrations::sqlitemigrations::MIGRATIONS,
    },
};
use rusqlite::{Connection, Error, ToSql, params, types::ToSqlOutput};
use uuid::Uuid;

impl ToSql for PaymentType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let val = match self {
            PaymentType::Income => 1,
            PaymentType::Outcome => 2,
            _ => 0,
        };
        Ok(ToSqlOutput::from(val))
    }
}
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

impl TransactionWorker for SqliteProvider {
    fn execute_transaction(
        &self,
        transaction: &MoneyTransaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let amount = match transaction.payment_type {
            PaymentType::Income => transaction.amount,
            PaymentType::Outcome => -transaction.amount,
            _ => 0.0,
        };
        self.change_money(&transaction.account, amount)?;

        let sql = "INSERT INTO Transactions VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
        let params = params![
            Uuid::new_v4().to_string(),
            transaction.amount,
            transaction.description.clone(),
            transaction.account.user_id,
            transaction.account.id,
            transaction.payment_type,
            transaction.payment_target.clone(),
            transaction.create_date.clone(),
        ];
        self.connection.execute(sql, params)?;
        Ok(())
    }
}

impl UserProvider for SqliteProvider {
    fn add_user(&self, user: &User) -> Result<(), Box<dyn std::error::Error>> {
        let sql = "insert into Users(Name, Number, CreationDate) values (?1,?2, ?3);";
        self.connection.execute(
            sql,
            params![user.name.clone(), user.number.clone(), user.creation_date,],
        )?;
        Ok(())
    }

    fn get_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
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

    fn get_user_by_number(&self, number: &str) -> Result<User, Box<dyn std::error::Error>> {
        let user = self.connection.query_one(
            "Select * from users where Number = ?1",
            [number],
            |row| {
                let user = User::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
                Ok(user)
            },
        )?;

        Ok(user)
    }

    fn delete_user_by_id(&self, id: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.connection
            .execute("Delete from Users where Id = ?1", [id])?;
        Ok(())
    }
}

impl AccountProvider for SqliteProvider {
    fn add_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error>> {
        let sql =
            "Insert into Accounts(Name, UserId, MoneyCount, CreationDate) Values (?1,?2,?3,?4);";
        self.connection.execute(
            sql,
            params![
                account.name.clone(),
                account.user_id,
                account.money,
                account.creation_date,
            ],
        )?;
        Ok(())
    }

    fn delete_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error>> {
        self.connection
            .execute("Delete from Accounts where Id = ?1", [account.id])?;
        Ok(())
    }

    fn change_money(
        &self,
        account: &Account,
        payment_count: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.execute(
            "Update Accounts set MoneyCount = ?2 where Id = ?1",
            params![account.id, (account.money + payment_count)],
        )?;
        Ok(())
    }

    fn get_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
        let mut values = self.connection.prepare("select * from Accounts;")?;
        let rows = values.query_map([], |row| {
            Ok(Account::from_exist(
                row.get(0)?,
                row.get(2)?,
                row.get(1)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?;

        let mut accounts: Vec<Account> = vec![];
        rows.into_iter().for_each(|user| match user {
            Ok(res) => accounts.push(res),
            Err(_) => (),
        });
        Ok(accounts)
    }

    fn search_account_by_user(&self, user: &User) -> Result<Account, Box<dyn std::error::Error>> {
        let account = self.connection.query_one(
            "Select * from Accounts where UserId = ?1",
            [user.id],
            |row| {
                Ok(Account::from_exist(
                    row.get(0)?,
                    row.get(2)?,
                    row.get(1)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )?;

        Ok(account)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tokio::fs;

    use crate::{
        config::Configuration,
        models::{
            account::Account,
            moneytransaction::{MoneyTransaction, PaymentType},
            user::User,
        },
        providers::{
            AccountProvider, TransactionWorker, UserProvider, bases::sqlite::SqliteProvider,
        },
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

    #[test]
    fn account_lifecircle_test() {
        let user = User::new(
            1,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        let sqlite_provider = configure_sql_with_user(&user);
        let account = Account::new(1, "Test".to_owned(), 50000.0);
        sqlite_provider.add_account(&account).unwrap();
        let account = sqlite_provider.search_account_by_user(&user).unwrap();
        sqlite_provider.change_money(&account, 100000.0).unwrap();
        let account = sqlite_provider.search_account_by_user(&user).unwrap();
        assert_eq!(account.money, 150000.0);
        assert_eq!(account.name, "Test");
        sqlite_provider.delete_account(&account).unwrap();
    }

    #[test]
    fn transaction_execute_test() {
        let user = User::new(
            1,
            String::from_str("scam").unwrap(),
            uuid::Uuid::new_v4().to_string(),
            String::from_str("2025-10-10").unwrap(),
        );
        let sqlite_provider = configure_sql_with_user(&user);
        let account = Account::new(1, "Test".to_owned(), 50000.0);
        sqlite_provider.add_account(&account).unwrap();
        let account = sqlite_provider.search_account_by_user(&user).unwrap();
        sqlite_provider
            .execute_transaction(&MoneyTransaction {
                description: "Test transcation".to_string(),
                amount: 200000.0,
                user: user.clone(),
                account: account.clone(),
                payment_type: PaymentType::Income,
                payment_target: "Test".to_string(),
                id: "".to_string(),
                create_date: chrono::Utc::now().naive_utc(),
            })
            .unwrap();
        let account = sqlite_provider.search_account_by_user(&user).unwrap();
        assert_eq!(account.money, 250000.0);
    }

    fn configure_sql_with_user(user: &User) -> SqliteProvider {
        let config = Configuration::memory_base();
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        sqlite_provider.add_user(user).unwrap();

        sqlite_provider
    }

    async fn check_exist(path: &str) -> bool {
        let meta = fs::metadata(path).await.ok();
        if let Some(_) = meta { true } else { false }
    }
}
