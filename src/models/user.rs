use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/*
Struct for user entry.
Id, indentifier.
name , user name.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub creation_date: NaiveDate,
}

impl User {
    pub fn new(id: i32, name: String, create_date: String) -> Self {
        let date = match NaiveDate::from_str(create_date.as_str()) {
            Ok(res) => res,
            Err(_) => chrono::Utc::now().naive_utc().date(),
        };
        Self {
            id: id,
            name: name.to_owned(),
            creation_date: date,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::user::User;

    #[test]
    fn create_user_test() {
        let date: String = "1999-01-01".to_owned();
        let id = 1;
        let name: String = "TestUser".to_owned();
        let user = User::new(id, name.clone(), date.clone());
        assert_eq!(user.creation_date.to_string(), date);
        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
    }

    #[test]
    fn create_user_failed_parse_date() {
        let user = User::new(1, "1".to_owned(), "20a5-01-01".to_owned());
        assert_eq!(
            user.creation_date.to_string(),
            chrono::Utc::now().date_naive().to_string()
        );
    }
}
