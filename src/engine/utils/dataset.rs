use crate::common::operation_error::{OperationError, OperationResult};
use arrow_array::RecordBatch;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Row;
use std::fs::File;
use std::path::Path;
