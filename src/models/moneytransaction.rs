use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{account::Account, user::User};
/*
Type for payment.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentType {
    None = 0,
    Income = 1,
    Outcome = 2,
}

/*
Payment record.
Contains information about user, payment type and payment count.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyTransaction {
    pub id: String,
    pub amount: f32,
    pub description: String,
    pub user: User,
    pub account: Account,
    pub payment_type: PaymentType,
    pub create_date: NaiveDateTime,
}
