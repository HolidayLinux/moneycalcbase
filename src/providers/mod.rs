use crate::{
    commands::{
        accounts::addaccountcommand::AddAccountCommand, users::addusercommand::AddUserCommand,
    },
    models::{account::Account, moneytransaction::MoneyTransaction, user::User},
};
use async_trait::async_trait;
use std::error;

pub mod bases;

#[async_trait]
pub trait DataProvider: UserProvider + AccountProvider + TransactionWorker {}

/// User provider interface.
/// Get functions for get or add users.
#[async_trait]
pub trait UserProvider: Send + Sync {
    async fn add_user(
        &self,
        add_user_command: &AddUserCommand,
    ) -> Result<(), Box<dyn error::Error>>;

    async fn get_users(&self) -> Result<Vec<User>, Box<dyn error::Error>>;

    async fn get_user_by_number(&self, number: &str) -> Result<User, Box<dyn error::Error>>;

    async fn delete_user_by_id(&self, id: i32) -> Result<(), Box<dyn error::Error>>;
}

/// Account provider interface.
/// Functionality for account actions.
#[async_trait]
pub trait AccountProvider: Send + Sync {
    async fn search_account_by_user(&self, user: &User) -> Result<Account, Box<dyn error::Error>>;

    async fn add_account(
        &self,
        add_account_command: &AddAccountCommand,
    ) -> Result<(), Box<dyn error::Error>>;

    async fn delete_account(&self, account: &Account) -> Result<(), Box<dyn error::Error>>;

    async fn change_money(
        &self,
        account: &Account,
        payment_count: f32,
    ) -> Result<(), Box<dyn error::Error>>;

    async fn get_accounts(&self) -> Result<Vec<Account>, Box<dyn error::Error>>;
}

/// Transaction Worker.
#[async_trait]
pub trait TransactionWorker: Send + Sync {
    async fn execute_transaction(
        &self,
        transaction: &MoneyTransaction,
    ) -> Result<(), Box<dyn error::Error>>;
}
