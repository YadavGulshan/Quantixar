use std::borrow::Cow;

use crate::common::operation_error::OperationError;
use crate::engine::types::tiny_kv;
use crate::engine::types::types::{DenseVector, VectorElementType};
use crate::engine::types::vector::{Vector, VectorRef};

pub type CowKey<'a> = Cow<'a, str>;

#[derive(Clone, PartialEq, Debug)]
pub enum CowVector<'a> {
  Dense(Cow<'a, [VectorElementType]>),
}


impl<'a> Default for CowVector<'a> {
  fn default() -> Self {
    CowVector::Dense(Cow::Owned(Vec::new()))
  }
}

pub(crate) type TinyMap<'a> = tiny_kv::TinyKV<CowKey<'a>, CowVector<'a>>;


impl<'a> CowVector<'a> {
  pub fn to_owned(self) -> Vector {
    match self {
      CowVector::Dense(v) => Vector::Dense(v.into_owned()),
    }
  }

  pub fn as_vec_ref(&self) -> VectorRef {
    match self {
      CowVector::Dense(v) => VectorRef::Dense(v.as_ref()),
    }
  }
}

impl<'a> From<Vector> for CowVector<'a> {
  fn from(v: Vector) -> Self {
    match v {
      Vector::Dense(v) => CowVector::Dense(Cow::Owned(v)),
    }
  }
}


impl<'a> From<DenseVector> for CowVector<'a> {
  fn from(v: DenseVector) -> Self {
    CowVector::Dense(Cow::Owned(v))
  }
}


impl<'a> From<&'a [VectorElementType]> for CowVector<'a> {
  fn from(v: &'a [VectorElementType]) -> Self {
    CowVector::Dense(Cow::Owned(v.into()))
  }
}


impl<'a> TryFrom<CowVector<'a>> for DenseVector {
  type Error = OperationError;

  fn try_from(value: CowVector<'a>) -> Result<Self, Self::Error> {
    match value {
      CowVector::Dense(v) => Ok(v.into_owned()),
    }
  }
}

impl<'a> From<VectorRef<'a>> for CowVector<'a> {
  fn from(v: VectorRef<'a>) -> Self {
    match v {
      VectorRef::Dense(v) => CowVector::Dense(Cow::Borrowed(v)),
    }
  }
}
