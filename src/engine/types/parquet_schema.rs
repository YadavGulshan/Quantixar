use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::SchemaRef;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use crate::common::operation_error::{OperationError, OperationResult};

#[derive(Clone)]
pub struct ParquetSchema {
    pub target_field: FieldInfo,
    pub identifier_field: FieldInfo,
}

#[derive(Clone)]
pub struct FieldInfo {
    pub name: String,
    pub data_type: String,
}

/// ParquetFile is a struct that represents a parquet file
/// and its schema.
/// parquet_schema will be used to dump the vectors into the index, and rest of the data will be used as metadata for a given vector.
#[derive(Clone)]
pub struct ParquetFile {
    pub file_path: String,
    pub parquet_schema: ParquetSchema,
    data: Option<Cow<'static, RecordBatch>>,
}

impl ParquetFile {
    pub fn new(file_path: String, parquet_schema: ParquetSchema) -> Self {
        let mut parquet_file = ParquetFile {
            file_path,
            parquet_schema,
            data: None,
        };

        parquet_file.init().unwrap();
        parquet_file
    }
    fn init(&mut self) -> OperationResult<()> {
        if self.data.is_none() {
            let data = self.read_parquet()?;
            self.data = Some(Cow::Owned(data.first()
                .ok_or_else(|| OperationError::ServiceError {
                    description: "No data found in parquet file".to_string(),
                    backtrace: None,
                })?.clone()
            ));
        }
        Ok(())
    }
    pub fn read_parquet(&self) -> OperationResult<Vec<RecordBatch>> {
        let path = Path::new(self.file_path.as_str());
        if !path.exists() {
            panic!("File not found: {}", self.file_path)
        }
        let file = File::open(path)?;
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)?
            .with_batch_size(8192)
            .build()?;

        let mut batches = Vec::new();
        for batch in reader {
            batches.push(batch?);
        }
        Ok(batches)
    }

    fn get_raw_schema(&self) -> OperationResult<SchemaRef> {
        if let Some(data) = self.data.as_ref() {
            Ok(data.schema())
        } else {
            Err(OperationError::ServiceError {
                description: "No data found in parquet file".to_string(),
                backtrace: None,
            })
        }
    }

    fn get_data(&self) -> Result<&Cow<'static, RecordBatch>, OperationError> {
        self.data
            .as_ref()
            .ok_or_else(|| OperationError::ServiceError {
                description: "No data found in parquet file".to_string(),
                backtrace: None,
            })
    }

    pub fn get_col_names(&self) -> Vec<String> {
        let binding = self.get_raw_schema().unwrap();
        let batches = binding.all_fields();
        let names = batches.iter().map(|x| {
            x.name().clone()
        }).collect::<Vec<String>>();
        return names;
    }

    pub fn get_columns(&self) -> OperationResult<Vec<ArrayRef>> {
        let cols = self.get_data()?
            .columns()
            .to_vec();
        Ok(cols)
    }

    /// use get_col_names, to get the idx of column, and then pull the data from get_cols
    pub fn target_col(&self) -> OperationResult<ArrayRef> {
        let target_col_idx = self.get_col_names().iter().position(
            |x| x == &self.parquet_schema.target_field.name
        ).ok_or_else(|| {
            OperationError::ServiceError {
                description: format!("Target column '{}' not found", self.parquet_schema.target_field.name),
                backtrace: None,
            }
        })?;

        let cols = self.get_columns()?;
        Ok(cols[target_col_idx].clone())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Binary;
    use std::sync::Arc;
    use arrow_array::{Array, BinaryArray, Float32Array, Float64Array, Int32Array};
    use crate::engine::types::parquet_schema::{FieldInfo, ParquetFile, ParquetSchema};

    fn text_to_f32(text: &str) -> Result<f32, std::num::ParseFloatError> {
        text.parse::<f32>()
    }

    #[test]
    fn test_read_parquet() {
        let parquet_file = ParquetFile::new(
            "assets/top_1000.parquet".to_string(),
            ParquetSchema {
                target_field: FieldInfo {
                    name: "vector".to_string(),
                    data_type: "float32".to_string(),
                },
                identifier_field: FieldInfo {
                    name: "id".to_string(),
                    data_type: "int32".to_string(),
                },
            },
        );
        let schema = parquet_file.get_raw_schema().unwrap();
        // dbg!(schema);

        let binary_data: Arc<dyn Array> = parquet_file.target_col().unwrap(); // Vectors are binary array
        let binary_array = binary_data.as_any()
            .downcast_ref::<BinaryArray>()
            .expect("Failed to cast as BinaryArray");

        let _ = binary_array.iter()
            .map(|value| {
                value.map(|bytes| {
                    dbg!(std::str::from_utf8(bytes).unwrap());
                })
            });
    }
}