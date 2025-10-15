use rusqlite::Error;

use crate::models::user::User;

pub mod bases;

/// User provider interface.
/// Get functions for get or add users.
trait UserProvider {
    fn add_user(&self, user: &User) -> Result<(), Error>;

    fn get_users(&self) -> Result<Vec<User>, Error>;
}
