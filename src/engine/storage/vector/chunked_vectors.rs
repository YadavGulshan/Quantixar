use std::cmp::max;
use std::collections::TryReserveError;

use crate::engine::types::types::PointOffsetType;
use crate::engine::utils::vectors::{TrySetCapacity, TrySetCapacityExact};

const CHUNK_SIZE: usize = 32 * 1024 * 1024;

// if dimension is too high, use this capacity
const MIN_CHUNK_CAPACITY: usize = 16;

#[derive(Debug)]
pub struct ChunkedVectors<T> {
    /// Vector's dimension.
    ///
    /// Each vector will consume `size_of::<T>() * dim` bytes.
    dim: usize,
    /// Number of stored vectors in all chunks.
    len: usize,
    /// Maximum number of vectors in each chunk.
    chunk_capacity: usize,
    chunks: Vec<Vec<T>>,
}

impl<T: Clone + Copy + Send + Sync + Default> ChunkedVectors<T> {
    pub fn new(dim: usize) -> Self {
        let chunk_capacity = CHUNK_SIZE / std::mem::size_of::<T>() / dim;
        let chunk_capacity = max(chunk_capacity, MIN_CHUNK_CAPACITY);
        Self {
            dim,
            len: 0,
            chunk_capacity,
            chunks: vec![Vec::with_capacity(chunk_capacity * dim)],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get<TKey>(&self, key: TKey) -> &[T]
    where
        TKey: num_traits::cast::AsPrimitive<usize>,
    {
        let key: usize = key.as_();
        let chunk_idx: &Vec<T> = &self.chunks[key / self.chunk_capacity];
        let idx: usize = (key % self.chunk_capacity) * self.dim;
        &chunk_idx[idx..idx + self.dim]
    }

    pub fn get_all_vectors(&self) -> Vec<(&[T], usize)> {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.chunks_exact(self.dim))
            .map(|vec| (vec, vec.len()))
            .collect::<Vec<_>>()
    }

    pub fn push(&mut self, vector: &[T]) -> Result<PointOffsetType, TryReserveError> {
        let new_id = self.len as PointOffsetType;
        self.insert(new_id, vector)?;
        Ok(new_id)
    }

    pub fn insert(&mut self, key: PointOffsetType, vector: &[T]) -> Result<(), TryReserveError> {
        let key = key as usize;

        self.len = max(self.len, key + 1);
        self.chunks
            .resize_with(self.len.div_ceil(self.chunk_capacity), Vec::new);

        let chunk_idx = key / self.chunk_capacity;
        let chunk_data = &mut self.chunks[chunk_idx];
        let idx = (key % self.chunk_capacity) * self.dim;

        // Grow the current chunk if needed to fit the new vector.
        //
        // All chunks are dynamically resized to fit their vectors in it.
        // Chunks have a size of zero by default. It's grown with zeroes to fit new vectors.
        //
        // The capacity for the first chunk is allocated normally to keep the memory footprint as
        // small as possible, see
        // <https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation>).
        // All other chunks allocate their capacity in full on first use to prevent expensive
        // reallocations when their data grows.
        if chunk_data.len() < idx + self.dim {
            if chunk_idx != 0 {
                let desired_capacity = self.chunk_capacity * self.dim;
                chunk_data.try_set_capacity_exact(desired_capacity)?;
            }
            chunk_data.resize_with(idx + self.dim, Default::default);
        }
        let data = &mut chunk_data[idx..idx + self.dim];
        log::debug!(
            "length of data: {} and vector: {}",
            data.len(),
            vector.len()
        );
        data.copy_from_slice(vector);
        Ok(())
    }
}

impl<T: Clone> TrySetCapacity for ChunkedVectors<T> {
    fn try_set_capacity(&mut self, capacity: usize) -> Result<(), TryReserveError> {
        let num_chunks = capacity.div_ceil(self.chunk_capacity);
        let last_chunk_idx = capacity / self.chunk_capacity;
        self.chunks.try_set_capacity_exact(num_chunks)?;
        self.chunks.resize_with(num_chunks, Vec::new);
        for chunk_idx in 0..num_chunks {
            if chunk_idx == last_chunk_idx {
                let desired_capacity = (capacity % self.chunk_capacity) * self.dim;
                self.chunks[chunk_idx].try_set_capacity_exact(desired_capacity)?;
            } else {
                let desired_capacity = self.chunk_capacity * self.dim;
                self.chunks[chunk_idx].try_set_capacity_exact(desired_capacity)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_vectors_basic_operations() {
        let mut vectors: ChunkedVectors<f32> = ChunkedVectors::new(3);
        assert_eq!(vectors.len(), 0);
        assert!(vectors.is_empty());

        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![4.0, 5.0, 6.0];
        let v3 = vec![7.0, 8.0, 9.0];

        let k1 = vectors.push(&v1).unwrap();
        let k2 = vectors.push(&v2).unwrap();
        let k3 = vectors.push(&v3).unwrap();

        assert_eq!(vectors.len(), 3);
        assert!(!vectors.is_empty());

        assert_eq!(vectors.get(k1), v1.as_slice());
        assert_eq!(vectors.get(k2), v2.as_slice());
        assert_eq!(vectors.get(k3), v3.as_slice());

        println!("all vectors: {:?}", vectors)
    }
    #[test]
    fn test_get_all_vectors() {
        let mut vectors: ChunkedVectors<f32> = ChunkedVectors::new(3);
        assert_eq!(vectors.len(), 0);
        assert!(vectors.is_empty());

        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![4.0, 5.0, 6.0];
        let v3 = vec![7.0, 8.0, 9.0];

        let k1 = vectors.push(&v1).unwrap();
        let k2 = vectors.push(&v2).unwrap();
        let k3 = vectors.push(&v3).unwrap();

        assert_eq!(vectors.len(), 3);
        assert!(!vectors.is_empty());

        assert_eq!(vectors.get(k1), v1.as_slice());
        assert_eq!(vectors.get(k2), v2.as_slice());
        assert_eq!(vectors.get(k3), v3.as_slice());

        println!("all vectors: {:?}", vectors.get_all_vectors())
    }
}
