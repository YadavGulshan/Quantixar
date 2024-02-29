use std::collections::HashMap;

use serde_json::Value;

use crate::{
    common::operation_error::OperationResult,
    engine::types::types::{Payload, PointOffsetType},
};

#[derive(Default)]
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
}
