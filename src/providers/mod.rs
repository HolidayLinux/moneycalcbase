use std::error;

use crate::models::{account::Account, moneytransaction::MoneyTransaction, user::User};

pub mod bases;

/// User provider interface.
/// Get functions for get or add users.
pub trait UserProvider {
    fn add_user(&self, user: &User) -> Result<(), Box<dyn error::Error>>;

    fn get_users(&self) -> Result<Vec<User>, Box<dyn error::Error>>;

    fn get_user_by_number(&self, number: &str) -> Result<User, Box<dyn error::Error>>;

    fn delete_user_by_id(&self, id: i32) -> Result<(), Box<dyn error::Error>>;
}

/// Account provider interface.
/// Functionality for account actions.
pub trait AccountProvider {
    fn search_account_by_user(&self, user: &User) -> Result<Account, Box<dyn error::Error>>;

    fn add_account(
        &self,
        add_account_command: AddAccountCommand,
    ) -> Result<Account, Box<dyn error::Error>>;

    fn delete_account(&self, account: &Account) -> Result<(), Box<dyn error::Error>>;

    fn change_money(
        &self,
        account: &Account,
        payment_count: f32,
    ) -> Result<(), Box<dyn error::Error>>;

    fn get_accounts(&self) -> Result<Vec<Account>, Box<dyn error::Error>>;
}

/// Transaction Worker.
pub trait TransactionWorker {
    fn execute_transaction(
        &self,
        transaction: &MoneyTransaction,
    ) -> Result<(), Box<dyn error::Error>>;
}
