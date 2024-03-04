use std::fs::File;
use std::path::Path;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use crate::common::operation_error::{OperationError, OperationResult};
use arrow_array::{RecordBatch};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Row;

