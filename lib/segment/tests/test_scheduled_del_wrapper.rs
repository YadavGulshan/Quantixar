#[allow(unused_imports)]
use crate::database::rocksdb_wrapper::tests::{DB_SCHEDULED_WRAPPER, DB_WRAPPER};

#[test]
fn test_put_and_remove() {
    let key1 = b"key1";
    let value1 = b"value1";

    let key2 = b"key2";
    let value2 = b"value2";

    // Putting a value and verifying it was added
    DB_SCHEDULED_WRAPPER.put(key1, value1).unwrap();
    DB_SCHEDULED_WRAPPER.put(key2, value2).unwrap();

    let val = DB_WRAPPER.get_pinned(key1, |v| v.to_vec()).unwrap();
    assert_eq!(val, Some(value1.to_vec()));

    // Removing a value and verifying it is scheduled for deletion but not yet deleted
    DB_SCHEDULED_WRAPPER.remove(key1).unwrap();
    assert_eq!(DB_SCHEDULED_WRAPPER.pending_delete_count(), 1);

    // Flushing and verifying the value is deleted
    let flush_function = DB_SCHEDULED_WRAPPER.flusher();
    flush_function().unwrap();
    let val = DB_WRAPPER.get_pinned(key1, |v| v.to_vec()).unwrap();
    assert_eq!(val, None);

    // Test 4: Verifying pending delete count is zero after flush
    assert_eq!(DB_SCHEDULED_WRAPPER.pending_delete_count(), 0);


    // Locking the DB and performing operations through LockedDatabaseColumnWrapper
    let locked_db = DB_SCHEDULED_WRAPPER.lock_db();
    let mut iter = locked_db.iter().unwrap();

    // Verify the iterator can find the key2-value2 pair
    if let Some((k, v)) = iter.next() {
        assert_eq!(&*k, key2);
        assert_eq!(&*v, value2);
    } else {
        panic!("Expected to find a key-value pair but found none");
    }
}

