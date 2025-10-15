use crate::models::user::User;
use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};
/*
Type for payment.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentType {
    Income,
    Outcome,
}

/*
Payment record.
Contains information about user, payment type and payment count.
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    id: String,
    amount: f64,
    description: String,
    payment_type: PaymentType,
    create_date: NaiveDateTime,
}
