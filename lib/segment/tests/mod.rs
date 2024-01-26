use lazy_static::lazy_static;
use rocksdb::DB;

use crate::database::rocksdb_wrapper::*;
use crate::database::rocksdb_wrapper::db_col_scheduled_del_wrapper::DatabaseColumnScheduledDeleteWrapper;
use crate::database::rocksdb_wrapper::db_column_wrapper::*;

mod test_col_wrapper;
mod test_db_col_iter;
mod test_scheduled_del_wrapper;

pub fn setup() -> (DatabaseColumnWrapper, DatabaseColumnScheduledDeleteWrapper) {
    let path = "/tmp/test_db";
    let db_opts = db_options();
    let cf_names = if check_db_exists(Path::new(path)) {
        DB::list_cf(&db_opts, path).unwrap()
    } else {
        vec!["default".to_string()] // Including default column family
    };

    let db = DB::open_cf(&db_opts, path, cf_names).unwrap();
    let database = Arc::new(RwLock::new(db));
    let db_wrapper = DatabaseColumnWrapper::new(database, "test_column");
    let db_scheduled_wrapper = DatabaseColumnScheduledDeleteWrapper::new(db_wrapper.clone());

    (db_wrapper, db_scheduled_wrapper)
}

lazy_static! {
    static ref SETUP: (DatabaseColumnWrapper, DatabaseColumnScheduledDeleteWrapper) = setup();
    static ref DB_WRAPPER: DatabaseColumnWrapper = SETUP.0.clone();
    static ref DB_SCHEDULED_WRAPPER: DatabaseColumnScheduledDeleteWrapper = SETUP.1.clone();
}
