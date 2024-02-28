use std::collections::HashMap;

use crate::engine::types::types::{Payload, PointOffsetType};

#[derive(Default)]
pub struct InMemoryPayloadStorage {
    pub(crate) payload: HashMap<PointOffsetType, Payload>,
}
