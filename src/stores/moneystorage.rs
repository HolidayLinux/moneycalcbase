use crate::{
    models::user::User,
    providers::{AccountProvider, TransactionWorker, UserProvider},
};

/// Interface for main storage with money data.
pub struct MoneyStorage<TU, TA, TW>
where
    TU: UserProvider,
    TA: AccountProvider,
    TW: TransactionWorker,
{
    user_provider: TU,
    account_provider: TA,
    transaction_worker: TW,
}

impl<TU: UserProvider, TA: AccountProvider, TW: TransactionWorker> MoneyStorage<TU, TA, TW> {
    pub fn new(user_provider: TU, account_provider: TA, transaction_worker: TW) -> Self {
        Self {
            user_provider,
            account_provider,
            transaction_worker,
        }
    }

    pub fn create_user(number: &str, name: &str) -> Result<User, Box<dyn std::error::Error>> {}
}
