pub struct StorageManagerOptions {
    fixed_value_bytes: i32,
}

impl StorageManagerOptions {
    pub fn new() -> StorageManagerOptions {
        StorageManagerOptions {
            fixed_value_bytes: -1,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.fixed_value_bytes != -1
    }

    pub fn to_string(&self) -> String {
        format!("{{fixed_value_bytes={}}}", self.fixed_value_bytes)
    }
}
