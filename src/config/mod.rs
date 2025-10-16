pub struct Configuration {
    pub connection_string: String,
    pub memory_base: bool,
}

impl Configuration {
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
