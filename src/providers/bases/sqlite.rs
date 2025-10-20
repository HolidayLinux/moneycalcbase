use core::error;
use std::sync::{Arc, Mutex};

use crate::{
    commands::{
        accounts::addaccountcommand::AddAccountCommand, users::addusercommand::AddUserCommand,
    },
    config::SqliteConfiguration,
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
use async_trait::async_trait;
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
    connection: Arc<Mutex<Connection>>,
    config: SqliteConfiguration,
}

impl Clone for SqliteProvider {
    fn clone(&self) -> Self {
        let connect: Connection;
        if self.config.memory_base {
            connect = Connection::open_in_memory().unwrap();
        } else {
            connect = Connection::open(self.config.connection_string.as_str()).unwrap();
        }

        Self {
            connection: Arc::new(Mutex::new(connect)),
            config: self.config.clone(),
        }
    }
}

impl SqliteProvider {
    pub fn new(
        config: &SqliteConfiguration,
        apply_migrations: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let connect: Connection;
        if config.memory_base {
            connect = Connection::open_in_memory()?;
        } else {
            connect = Connection::open(config.connection_string.as_str())?;
        }

        if apply_migrations {
            let mut mutcon = connect;

            MIGRATIONS.to_latest(&mut mutcon).unwrap();

            Ok(Self {
                connection: Arc::new(Mutex::new(mutcon)),
                config: config.clone(),
            })
        } else {
            Ok(Self {
                connection: Arc::new(Mutex::new(connect)),
                config: config.clone(),
            })
        }
    }

    fn execute_query<F, T>(&self, query: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce(&Connection) -> Result<T, Box<dyn std::error::Error>>,
    {
        let connection = self.connection.lock().map_err(|e| e.to_string())?;

        query(&connection)
    }
}

#[async_trait]
impl TransactionWorker for SqliteProvider {
    async fn execute_transaction(
        &self,
        transaction: &MoneyTransaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let amount = match transaction.payment_type {
            PaymentType::Income => transaction.amount,
            PaymentType::Outcome => -transaction.amount,
            _ => 0.0,
        };
        self.change_money(&transaction.account, amount).await?;

        self.execute_query(|connection| {
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
            connection.execute(sql, params)?;
            Ok(())
        })
    }
}

#[async_trait]
impl UserProvider for SqliteProvider {
    async fn add_user(
        &self,
        add_user_command: &AddUserCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            let sql = "insert into Users(Name, Number, CreationDate) values (?1,?2, ?3);";
            connection.execute(
                sql,
                params![
                    add_user_command.user_name,
                    add_user_command.user_number,
                    chrono::Utc::now().naive_utc().date(),
                ],
            )?;
            Ok(())
        })
    }

    async fn get_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            let mut values = connection.prepare("select * from Users;")?;
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
        })
    }

    async fn get_user_by_number(&self, number: &str) -> Result<User, Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            let user =
                connection.query_one("Select * from users where Number = ?1", [number], |row| {
                    let user = User::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
                    Ok(user)
                })?;

            Ok(user)
        })
    }

    async fn delete_user_by_id(&self, id: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            connection.execute("Delete from Users where Id = ?1", [id])?;
            Ok(())
        })
    }
}

#[async_trait]
impl AccountProvider for SqliteProvider {
    async fn add_account(
        &self,
        add_command: &AddAccountCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            let sql =
                "Insert into Accounts(Name, UserId, MoneyCount, CreationDate) Values (?1,?2,?3,?4);";
            connection.execute(
                sql,
                params![
                    add_command.account_name,
                    add_command.user_id,
                    add_command.initial_balance,
                    chrono::Utc::now().naive_utc().date().to_string(),
                ],
            )?;
            Ok(())
        })
    }

    async fn delete_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            connection.execute("Delete from Accounts where Id = ?1", [account.id])?;
            Ok(())
        })
    }

    async fn change_money(
        &self,
        account: &Account,
        payment_count: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            connection.execute(
                "Update Accounts set MoneyCount = ?2 where Id = ?1",
                params![account.id, (account.money + payment_count)],
            )?;
            Ok(())
        })
    }

    async fn get_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            let mut values = connection.prepare("select * from Accounts;")?;
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
        })
    }

    async fn search_account_by_user(
        &self,
        user: &User,
    ) -> Result<Account, Box<dyn std::error::Error>> {
        self.execute_query(|connection| {
            let account = connection.query_one(
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
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tokio::fs;

    use crate::{
        commands::{
            accounts::addaccountcommand::AddAccountCommand, users::addusercommand::AddUserCommand,
        },
        config::SqliteConfiguration,
        models::moneytransaction::{MoneyTransaction, PaymentType},
        providers::{
            AccountProvider, TransactionWorker, UserProvider, bases::sqlite::SqliteProvider,
        },
    };

    #[tokio::test]
    async fn create_test_file_base() {
        let config = SqliteConfiguration::new("./testbases/testbase.db3");
        let _ = SqliteProvider::new(&config, false);
        assert!(check_exist(config.connection_string.as_str()).await);
    }

    #[test]
    fn create_test_memory_base() {
        let config = SqliteConfiguration::memory_base();
        let _ = SqliteProvider::new(&config, false).unwrap();
    }

    #[tokio::test]
    async fn add_migration_test() {
        let config = SqliteConfiguration::new("./testbases/testbase_mig.db3");
        let _ = SqliteProvider::new(&config, true);
        assert!(check_exist(config.connection_string.as_str()).await);
    }

    #[tokio::test]
    async fn add_user_test() {
        let config = SqliteConfiguration::memory_base();
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        sqlite_provider.add_user(&add_user_command).await.unwrap();
        let user = sqlite_provider
            .get_user_by_number(add_user_command.user_number.as_str())
            .await
            .unwrap();

        sqlite_provider.delete_user_by_id(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn add_user_notunique_test() {
        let config = SqliteConfiguration::new("./testbases/testbase_user.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: String::from_str("88005553535").unwrap(),
        };

        assert!(sqlite_provider.add_user(&add_user_command).await.is_err());
    }

    #[tokio::test]
    async fn get_users_test() {
        let config = SqliteConfiguration::new("./testbases/testbase_users.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        sqlite_provider.add_user(&add_user_command).await.unwrap();
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scamjr").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        sqlite_provider.add_user(&add_user_command).await.unwrap();

        let users = sqlite_provider.get_users().await.unwrap();
        assert!(users.len() > 0);

        fs::remove_file("./testbases/testbase_users.db3")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_user_by_number_test() {
        let config = SqliteConfiguration::new("./testbases/testbase_user.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let user_res = sqlite_provider.get_user_by_number("88005553535").await;
        match user_res {
            Ok(user) => {
                assert!(user.name.eq("scam"));
            }
            Err(err) => {
                panic!("{}", err)
            }
        }
    }

    #[tokio::test]
    async fn add_account_to_db() {
        let config = SqliteConfiguration::new("./testbases/testbase_accounts.db3");
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        sqlite_provider.add_user(&add_user_command).await.unwrap();
        let add_account_command = create_add_account_command(1, 50000.0);
        sqlite_provider
            .add_account(&add_account_command)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_users_test() {
        let config = SqliteConfiguration::new("./testbases/testbase_user_delete.db3");
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();

        sqlite_provider.add_user(&add_user_command).await.unwrap();
        let user = sqlite_provider
            .get_user_by_number(add_user_command.user_number.as_str())
            .await
            .unwrap();

        let add_user_command = AddUserCommand {
            user_name: String::from_str("scamer").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        sqlite_provider.add_user(&add_user_command).await.unwrap();

        let users = sqlite_provider.get_users().await.unwrap();
        assert!(users.len() > 0);

        for user in users {
            sqlite_provider.delete_user_by_id(user.id).await.unwrap();
        }
        let users = sqlite_provider.get_users().await.unwrap();

        assert!(users.len() == 0);

        std::fs::remove_file("./testbases/testbase_user_delete.db3").unwrap();
    }

    #[tokio::test]
    async fn account_lifecircle_test() {
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        let sqlite_provider = configure_sql_with_user(&add_user_command).await;
        let user = sqlite_provider
            .get_user_by_number(add_user_command.user_number.as_str())
            .await
            .unwrap();
        let add_account_command = create_add_account_command(1, 50000.0);
        sqlite_provider
            .add_account(&add_account_command)
            .await
            .unwrap();
        let account = sqlite_provider.search_account_by_user(&user).await.unwrap();
        sqlite_provider
            .change_money(&account, 100000.0)
            .await
            .unwrap();
        let account = sqlite_provider.search_account_by_user(&user).await.unwrap();
        assert_eq!(account.money, 150000.0);
        assert_eq!(account.name, add_account_command.account_name);
        sqlite_provider.delete_account(&account).await.unwrap();
    }

    #[tokio::test]
    async fn transaction_execute_test() {
        let add_user_command = AddUserCommand {
            user_name: String::from_str("scam").unwrap(),
            user_number: uuid::Uuid::new_v4().to_string(),
        };
        let sqlite_provider = configure_sql_with_user(&add_user_command).await;
        let user = sqlite_provider
            .get_user_by_number(add_user_command.user_number.as_str())
            .await
            .unwrap();
        let add_account_command = create_add_account_command(1, 50000.0);
        sqlite_provider
            .add_account(&add_account_command)
            .await
            .unwrap();
        let account = sqlite_provider.search_account_by_user(&user).await.unwrap();
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
            .await
            .unwrap();
        let account = sqlite_provider.search_account_by_user(&user).await.unwrap();
        assert_eq!(account.money, 250000.0);
    }

    async fn configure_sql_with_user(add_user_command: &AddUserCommand) -> SqliteProvider {
        let config = SqliteConfiguration::memory_base();
        let sqlite_provider = SqliteProvider::new(&config, true).unwrap();
        sqlite_provider.add_user(add_user_command).await.unwrap();

        sqlite_provider
    }

    async fn check_exist(path: &str) -> bool {
        let meta = fs::metadata(path).await.ok();
        if let Some(_) = meta { true } else { false }
    }

    fn create_add_account_command(user_id: i32, initial_balance: f32) -> AddAccountCommand {
        AddAccountCommand {
            user_id,
            account_name: String::from_str("TEST ACCOUNT").unwrap(),
            initial_balance,
        }
    }
}
