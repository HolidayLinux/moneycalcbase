use crate::providers::{
    AccountProvider, TransactionWorker, UserProvider, bases::sqlite::SqliteProvider,
};

#[derive(Clone, Debug)]
pub struct SqliteConfiguration {
    pub connection_string: String,
    pub memory_base: bool,
}

pub trait StorageConfiguration<T>
where
    T: UserProvider + AccountProvider + TransactionWorker,
{
    fn configure(&self) -> Result<T, Box<dyn std::error::Error>>;
}

impl SqliteConfiguration {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            memory_base: false,
        }
    }

    pub fn memory_base() -> Self {
        Self {
            connection_string: String::new(),
            memory_base: true,
        }
    }
}

impl StorageConfiguration<SqliteProvider> for SqliteConfiguration {
    fn configure(&self) -> Result<SqliteProvider, Box<dyn std::error::Error>> {
        SqliteProvider::new(&self, true)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{SqliteConfiguration, StorageConfiguration};

    #[test]
    fn create_storage_by_config() {
        let config = SqliteConfiguration::memory_base();

        let _ = config.configure().unwrap();
    }

    #[test]
    fn create_storage_by_config_path() {
        let config = SqliteConfiguration::new("test.db");

        let _ = config.configure().unwrap();

        let res = std::fs::exists("test.db").unwrap();
        assert!(res);
        std::fs::remove_file("test.db").unwrap();
        let res = std::fs::exists("test.db").unwrap();
        assert!(!res);
        let res = std::fs::remove_file("test.db").is_err();
        assert!(res);
    }
}
