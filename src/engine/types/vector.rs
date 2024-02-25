use std::borrow::Cow;
use std::collections::HashMap;

use axum::extract::FromRef;
use procfs::WithCurrentSystemInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::common;
use crate::common::operation_error::OperationError;
use crate::engine::types::cow_vector::CowVector;
use crate::engine::types::named_vector::NamedVectors;
use crate::engine::types::types::{DEFAULT_VECTOR_NAME, DenseVector, VectorElementType};
use crate::engine::utils::named_vector::transpose_map_into_named_vector;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Vector {
  Dense(DenseVector),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VectorRef<'a> {
  Dense(&'a [VectorElementType]),
}

impl Vector {
  pub fn to_vec_ref(&self) -> VectorRef {
    match self {
      Vector::Dense(v) => VectorRef::Dense(v.as_slice()),
    }
  }
}

impl Validate for Vector {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    match self {
      Vector::Dense(_) => Ok(()),
    }
  }
}

impl<'a> VectorRef<'a> {
  pub fn to_vec(self) -> Vector {
    match self {
      VectorRef::Dense(v) => Vector::Dense(v.to_vec()),
    }
  }
}

impl<'a> TryFrom<VectorRef<'a>> for &'a [VectorElementType] {
  type Error = OperationError;

  fn try_from(value: VectorRef<'a>) -> Result<Self, Self::Error> {
    match value {
      VectorRef::Dense(v) => Ok(v),
    }
  }
}


impl From<NamedVectorStruct> for Vector {
  fn from(value: NamedVectorStruct) -> Self {
    match value {
      NamedVectorStruct::Default(v) => Vector::Dense(v),
      NamedVectorStruct::Dense(v) => Vector::Dense(v.vector),
    }
  }
}

impl TryFrom<Vector> for DenseVector {
  type Error = OperationError;

  fn try_from(value: Vector) -> Result<Self, Self::Error> {
    match value {
      Vector::Dense(v) => Ok(v),
    }
  }
}


impl<'a> From<&'a [VectorElementType]> for VectorRef<'a> {
  fn from(val: &'a [VectorElementType]) -> Self {
    VectorRef::Dense(val)
  }
}

impl<'a> From<&'a DenseVector> for VectorRef<'a> {
  fn from(val: &'a DenseVector) -> Self {
    VectorRef::Dense(val.as_slice())
  }
}


impl From<DenseVector> for Vector {
  fn from(val: DenseVector) -> Self {
    Vector::Dense(val)
  }
}


impl<'a> From<&'a Vector> for VectorRef<'a> {
  fn from(val: &'a Vector) -> Self {
    match val {
      Vector::Dense(v) => VectorRef::Dense(v.as_slice()),
    }
  }
}

impl<'a> VectorRef<'a> {
  // Cannot use `ToOwned` trait because of `Borrow` implementation for `Vector`
  pub fn to_owned(self) -> Vector {
    match self {
      VectorRef::Dense(v) => Vector::Dense(v.to_vec()),
    }
  }

  pub fn len(&self) -> usize {
    match self {
      VectorRef::Dense(v) => v.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl<'a> TryInto<&'a [VectorElementType]> for &'a Vector {
  type Error = OperationError;

  fn try_into(self) -> Result<&'a [VectorElementType], Self::Error> {
    match self {
      Vector::Dense(v) => Ok(v),
    }
  }
}


pub fn default_vector(vec: Vec<VectorElementType>) -> NamedVectors<'static> {
  NamedVectors::from([(DEFAULT_VECTOR_NAME.to_owned(), vec)])
}

pub fn only_default_vector(vec: &[VectorElementType]) -> NamedVectors {
  NamedVectors::from_ref(DEFAULT_VECTOR_NAME, vec.into())
}

/// Full vector data per point separator with single and multiple vector modes
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, rename_all = "snake_case")]
pub enum VectorStruct {
  Single(DenseVector),
  Multi(HashMap<String, Vector>),
}

impl VectorStruct {
  /// Check if this vector struct is empty.
  pub fn is_empty(&self) -> bool {
    match self {
      VectorStruct::Single(vector) => vector.is_empty(),
      VectorStruct::Multi(vectors) => vectors.values().all(|v| match v {
        Vector::Dense(vector) => vector.is_empty(),
      }),
    }
  }
}

impl Validate for VectorStruct {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    match self {
      VectorStruct::Single(_) => Ok(()),
      VectorStruct::Multi(v) => common::validation::validate_iter(v.values()),
    }
  }
}

impl From<DenseVector> for VectorStruct {
  fn from(v: DenseVector) -> Self {
    VectorStruct::Single(v)
  }
}

impl From<&[VectorElementType]> for VectorStruct {
  fn from(v: &[VectorElementType]) -> Self {
    VectorStruct::Single(v.to_vec())
  }
}

impl<'a> From<NamedVectors<'a>> for VectorStruct {
  fn from(v: NamedVectors) -> Self {
    if v.len() == 1 && v.contains_key(DEFAULT_VECTOR_NAME) {
      let vector: &[_] = v.get(DEFAULT_VECTOR_NAME).unwrap().try_into().unwrap();
      VectorStruct::Single(vector.to_owned())
    } else {
      VectorStruct::Multi(v.into_owned_map())
    }
  }
}

impl VectorStruct {
  pub fn get(&self, name: &str) -> Option<VectorRef> {
    match self {
      VectorStruct::Single(v) => (name == DEFAULT_VECTOR_NAME).then_some(v.into()),
      VectorStruct::Multi(v) => v.get(name).map(|v| v.into()),
    }
  }

  pub fn into_all_vectors(self) -> NamedVectors<'static> {
    match self {
      VectorStruct::Single(v) => default_vector(v),
      VectorStruct::Multi(v) => NamedVectors::from_map(v),
    }
  }
}

/// Vector data with name
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub struct NamedVector {
  /// Name of vector data
  pub name: String,
  /// Vector data
  pub vector: DenseVector,
}

/// Sparse vector data with name
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Validate)]
#[serde(rename_all = "snake_case")]
pub struct NamedSparseVector {
  /// Name of vector data
  pub name: String,
}

/// Vector data separator for named and unnamed modes
/// Unnamed mode:
///
/// {
///   "vector": [1.0, 2.0, 3.0]
/// }
///
/// or named mode:
///
/// {
///   "vector": {
///     "vector": [1.0, 2.0, 3.0],
///     "name": "image-embeddings"
///   }
/// }
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum NamedVectorStruct {
  Default(DenseVector),
  Dense(NamedVector),
}

impl From<DenseVector> for NamedVectorStruct {
  fn from(v: DenseVector) -> Self {
    NamedVectorStruct::Default(v)
  }
}

impl From<NamedVector> for NamedVectorStruct {
  fn from(v: NamedVector) -> Self {
    NamedVectorStruct::Dense(v)
  }
}


pub trait Named {
  fn get_name(&self) -> &str;
}

impl Named for NamedVectorStruct {
  fn get_name(&self) -> &str {
    match self {
      NamedVectorStruct::Default(_) => DEFAULT_VECTOR_NAME,
      NamedVectorStruct::Dense(v) => &v.name,
    }
  }
}

impl NamedVectorStruct {
  pub fn new_from_vector(vector: Vector, name: String) -> Self {
    match vector {
      Vector::Dense(vector) => NamedVectorStruct::Dense(NamedVector { name, vector }),
    }
  }

  pub fn get_vector(&self) -> VectorRef {
    match self {
      NamedVectorStruct::Default(v) => v.as_slice().into(),
      NamedVectorStruct::Dense(v) => v.vector.as_slice().into(),
    }
  }

  pub fn to_vector(self) -> Vector {
    match self {
      NamedVectorStruct::Default(v) => v.into(),
      NamedVectorStruct::Dense(v) => v.vector.into(),
    }
  }
}

impl Validate for NamedVectorStruct {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    match self {
      NamedVectorStruct::Default(_) => Ok(()),
      NamedVectorStruct::Dense(_) => Ok(()),
    }
  }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum BatchVectorStruct {
  Single(Vec<DenseVector>),
  Multi(HashMap<String, Vec<Vector>>),
}

impl From<Vec<DenseVector>> for BatchVectorStruct {
  fn from(v: Vec<DenseVector>) -> Self {
    BatchVectorStruct::Single(v)
  }
}

impl BatchVectorStruct {
  pub fn into_all_vectors(self, num_records: usize) -> Vec<NamedVectors<'static>> {
    match self {
      BatchVectorStruct::Single(vectors) => vectors.into_iter().map(default_vector).collect(),
      BatchVectorStruct::Multi(named_vectors) => {
        if named_vectors.is_empty() {
          vec![NamedVectors::default(); num_records]
        } else {
          transpose_map_into_named_vector(named_vectors)
        }
      }
    }
  }
}

impl Validate for BatchVectorStruct {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    match self {
      BatchVectorStruct::Single(_) => Ok(()),
      BatchVectorStruct::Multi(v) => {
        common::validation::validate_iter(v.values().flat_map(|batch| batch.iter()))
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct NamedQuery<TQuery> {
  pub query: TQuery,
  pub using: Option<String>,
}

impl<T> Named for NamedQuery<T> {
  fn get_name(&self) -> &str {
    self.using.as_deref().unwrap_or(DEFAULT_VECTOR_NAME)
  }
}

impl<T: Validate> Validate for NamedQuery<T> {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    self.query.validate()
  }
}

#[derive(Debug, Clone)]
pub enum QueryVector {
  Nearest(Vector),
  // Recommend(RecoQuery<Vector>),
  // Discovery(DiscoveryQuery<Vector>),
  // Context(ContextQuery<Vector>),
}

impl From<DenseVector> for QueryVector {
  fn from(vec: DenseVector) -> Self {
    Self::Nearest(Vector::Dense(vec))
  }
}

impl<'a> From<&'a [VectorElementType]> for QueryVector {
  fn from(vec: &'a [VectorElementType]) -> Self {
    Self::Nearest(Vector::Dense(vec.to_vec()))
  }
}

impl<const N: usize> From<[VectorElementType; N]> for QueryVector {
  fn from(vec: [VectorElementType; N]) -> Self {
    let vec: VectorRef = vec.as_slice().into();
    Self::Nearest(vec.to_owned())
  }
}

impl<'a> From<VectorRef<'a>> for QueryVector {
  fn from(vec: VectorRef<'a>) -> Self {
    Self::Nearest(vec.to_vec())
  }
}

impl From<Vector> for QueryVector {
  fn from(vec: Vector) -> Self {
    Self::Nearest(vec)
  }
}

impl Into<Vector> for QueryVector {
  fn into(self) -> Vector {
    match self {
      QueryVector::Nearest(v) => v,
    }
  }
}
