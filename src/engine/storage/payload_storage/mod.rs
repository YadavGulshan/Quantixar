use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    common::operation_error::OperationResult,
    engine::types::types::{Payload, PointOffsetType},
};

#[derive(Default, Serialize, Deserialize)]
pub struct PayloadStorage {
    pub(crate) payload: HashMap<PointOffsetType, Payload>,
}

impl PayloadStorage {
    pub fn assign(&mut self, point_id: PointOffsetType, payload: &Payload) -> OperationResult<()> {
        match self.payload.get_mut(&point_id) {
            Some(point_payload) => point_payload.merge(payload),
            None => {
                self.payload.insert(point_id, payload.to_owned());
            }
        }
        Ok(())
    }

    pub fn payload(&self, point_id: PointOffsetType) -> OperationResult<Payload> {
        match self.payload.get(&point_id) {
            Some(payload) => Ok(payload.to_owned()),
            None => Ok(Default::default()),
        }
    }

    pub fn delete<'a>(
        &mut self,
        point_id: PointOffsetType,
        key: &'a str,
    ) -> OperationResult<Vec<Value>> {
        match self.payload.get_mut(&point_id) {
            Some(payload) => {
                let res = payload.remove(key);
                Ok(res)
            }
            None => Ok(vec![]),
        }
    }

    pub fn drop(&mut self, point_id: PointOffsetType) -> OperationResult<Option<Payload>> {
        let res = self.payload.remove(&point_id);
        Ok(res)
    }

    pub fn wipe(&mut self) -> OperationResult<()> {
        self.payload = HashMap::new();
        Ok(())
    }
    // To dump the struct into a binary file:
    pub fn dump_to_file(&self, file: &str) -> OperationResult<()> {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut file = File::create(file)?;
        file.write_all(&encoded)?;
        Ok(())
    }
    // To load the struct from a binary file:
    pub fn load_from_file(filename: &str) -> std::io::Result<PayloadStorage> {
        let mut file = File::open(filename).unwrap();
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded)?;
        let storage: PayloadStorage = bincode::deserialize(&encoded).unwrap();
        Ok(storage)
    }
}
