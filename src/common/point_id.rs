use std::{fmt::Formatter, str::FromStr};

use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, JsonSchema)]
#[serde(untagged)]
pub enum ExtendedPointId {
    NumId(u64),
    Uuid(Uuid),
}

impl std::fmt::Display for ExtendedPointId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtendedPointId::NumId(idx) => write!(f, "{idx}"),
            ExtendedPointId::Uuid(uuid) => write!(f, "{uuid}"),
        }
    }
}

impl From<u64> for ExtendedPointId {
    fn from(idx: u64) -> Self {
        ExtendedPointId::NumId(idx)
    }
}

impl FromStr for ExtendedPointId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let try_num: Result<u64, _> = s.parse();
        if let Ok(num) = try_num {
            return Ok(Self::NumId(num));
        }
        let try_uuid = Uuid::from_str(s);
        if let Ok(uuid) = try_uuid {
            return Ok(Self::Uuid(uuid));
        }
        Err(())
    }
}

impl<'de> serde::Deserialize<'de> for ExtendedPointId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = match serde_value::Value::deserialize(deserializer) {
            Ok(val) => val,
            Err(err) => return Err(err),
        };

        if let Ok(num) = value.clone().deserialize_into() {
            return Ok(ExtendedPointId::NumId(num));
        }

        if let Ok(uuid) = value.clone().deserialize_into() {
            return Ok(ExtendedPointId::Uuid(uuid));
        }

        Err(serde::de::Error::custom(format!(
            "value {} is not a valid point ID, \
             valid values are either an unsigned integer or a UUID",
            crate::utils::fmt::SerdeValue(&value),
        )))
    }
}

/// Type of point index across all segments
pub type PointIdType = ExtendedPointId;
