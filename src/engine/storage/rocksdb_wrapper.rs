#[cfg(feature = "rock")]
extern crate rocksdb;

use std::sync::Arc;

use parking_lot::RwLock;
use rocksdb::ColumnFamily;
#[cfg(feature = "rock")]
use rocksdb::DB;

use crate::common::operation_error::{
    OperationError,
    OperationResult,
};

#[derive(Clone)]
pub struct DatabaseColumnWrapper
{
    pub database: Arc<RwLock<DB>>,
    pub column_name: String,
}

pub struct DatabaseColumnIterator<'a>
{
    pub handle: &'a ColumnFamily,
    pub iter: rocksdb::DBRawIterator<'a>,
}

pub struct LockedDatabaseColumnWrapper<'a>
{
    pub(crate) guard: parking_lot::RwLockReadGuard<'a, DB>,
    pub(crate) column_name: &'a str,
}

impl<'a> LockedDatabaseColumnWrapper<'a>
{
    pub fn iter(&self) -> OperationResult<DatabaseColumnIterator>
    {
        DatabaseColumnIterator::new(&self.guard, self.column_name)
    }
}

impl<'a> DatabaseColumnIterator<'a>
{
    pub fn new(db: &'a DB, column_name: &str) -> OperationResult<DatabaseColumnIterator<'a>>
    {
        let handle = db.cf_handle(column_name).ok_or_else(|| {
            OperationError::service_error(format!(
                "RocksDB cf_handle error: Cannot find column family {column_name}"
            ))
        })?;
        let mut iter = db.raw_iterator_cf(&handle);
        iter.seek_to_first();
        Ok(DatabaseColumnIterator { handle, iter })
    }
}

impl<'a> Iterator for DatabaseColumnIterator<'a>
{
    type Item = (Box<[u8]>, Box<[u8]>);

    fn next(&mut self) -> Option<Self::Item>
    {
        // Stop if iterator has ended or errored
        if !self.iter.valid() {
            return None;
        }

        let item = (
            Box::from(self.iter.key().unwrap()),
            Box::from(self.iter.value().unwrap()),
        );

        // Search to next item for next iteration
        self.iter.next();

        Some(item)
    }
}
