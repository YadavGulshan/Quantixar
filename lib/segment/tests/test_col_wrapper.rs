#[allow(unused_imports)]
use crate::database::rocksdb_wrapper::tests::*;

fn byte_slice_to_string(slice: &[u8]) -> String {
    String::from_utf8(slice.to_vec()).unwrap()
}

#[test]
fn test_put_success() {
    let db_wrapper = DB_WRAPPER.clone();
    assert!(db_wrapper.put(b"key1", b"value1").is_ok());
    let val = db_wrapper.get_pinned(b"key1", byte_slice_to_string).unwrap();
    assert_eq!(val, Some("value1".to_string()));

    // Test overwrite
    assert!(db_wrapper.put(b"key1", b"value2").is_ok());
    let val = db_wrapper.get_pinned(b"key1", byte_slice_to_string).unwrap();
    assert_eq!(val, Some("value2".to_string()));
}

#[test]
fn test_flusher() {
    test_put_success();
    let db_wrapper = DB_WRAPPER.clone();
    let flush_function = db_wrapper.flusher();
    let flush_op = flush_function();
    assert!(flush_op.is_ok());
    let val = db_wrapper.get_pinned(b"key1", byte_slice_to_string).unwrap();
    println!("{:?}", val);
}