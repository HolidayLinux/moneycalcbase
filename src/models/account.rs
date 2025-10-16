use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Account type.
/// id for identification in base.
/// user_id for id of user.
/// name of account.
/// money is money count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub money: f32,
    pub creation_date: NaiveDate,
}

impl Account {
    pub fn new(user_id: i32, name: String, money: f32) -> Self {
        Self {
            id: 0,
            user_id,
            name: name.to_owned(),
            money,
            creation_date: chrono::Utc::now().naive_utc().date(),
        }
    }

    pub fn from_exist(
        id: i32,
        user_id: i32,
        name: String,
        money: f32,
        creation_date: NaiveDate,
    ) -> Self {
        Self {
            id,
            user_id,
            name,
            money,
            creation_date,
        }
    }
}
