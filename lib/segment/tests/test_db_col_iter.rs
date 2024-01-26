#![allow(unused_imports)]

use crate::database::rocksdb_wrapper::db_column_iter::DatabaseColumnIterator;
use crate::database::rocksdb_wrapper::hof::filter_keys;
use crate::database::rocksdb_wrapper::locked_db_col_wrapper::LockedDatabaseColumnWrapper;
use crate::database::rocksdb_wrapper::tests::DB_WRAPPER;

#[test]
fn test_filter_keys() {
    let db_wrapper = DB_WRAPPER.clone();

    db_wrapper.put(b"prefix_key1", b"value1").unwrap();
    db_wrapper.put(b"prefix_key2", b"value2").unwrap();
    db_wrapper.put(b"other_key3", b"value3").unwrap();

    let db_read = db_wrapper.database.read();
    let mut iter = DatabaseColumnIterator::new(&db_read, &db_wrapper.column_name).unwrap();

    let filtered_keys = filter_keys(&mut iter, |key| key.starts_with(b"prefix"));

    // Assert the results
    assert_eq!(
        filtered_keys,
        vec![
            b"prefix_key1".to_vec().into_boxed_slice(),
            b"prefix_key2".to_vec().into_boxed_slice()
        ]
    );
}

#[test]
fn test_database_column_iterator() {
    let db_wrapper = DB_WRAPPER.clone();
    db_wrapper.put(b"key1", b"value1").unwrap();
    db_wrapper.put(b"key2", b"value2").unwrap();
    db_wrapper.put(b"key3", b"value3").unwrap();
    let locked_db_col_wrapper = LockedDatabaseColumnWrapper {
        guard: db_wrapper.database.read(),
        column_name: &db_wrapper.column_name,
    };
    let mut iter = locked_db_col_wrapper.iter().unwrap();
    assert_eq!(
        iter.next().unwrap(),
        (
            b"key1".to_vec().into_boxed_slice(),
            b"value1".to_vec().into_boxed_slice()
        )
    );
}
