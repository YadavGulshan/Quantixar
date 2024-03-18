use std::{fs::File, os::fd::AsRawFd};

#[cfg(target_os = "linux")]
use io_uring::{opcode, types::Fd, IoUring};

use memory::mmap_ops::transmute_from_u8_to_slice;

use crate::{
    common::operation_error::{OperationError, OperationResult},
    engine::types::types::{PointOffsetType, VectorElementType},
};

const DISK_PARALLELISM: usize = 16; // TODO: benchmark it better, or make it configurable

struct BufferMeta {
    /// Sequential index of the processing point
    pub index: usize,
    /// Id of the point that is currently being processed
    pub point_id: PointOffsetType,
}

struct Buffer {
    /// Stores the buffer for the point vectors
    pub buffer: Vec<u8>,
    /// Stores the point ids that are currently being processed in each buffer.
    pub meta: Option<BufferMeta>,
}

pub struct BufferStore {
    /// Stores the buffer for the point vectors
    buffers: Vec<Buffer>,
}

impl BufferStore {
    pub fn new(num_buffers: usize, buffer_raw_size: usize) -> Self {
        Self {
            buffers: (0..num_buffers)
                .map(|_| Buffer {
                    buffer: vec![0; buffer_raw_size],
                    meta: None,
                })
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self { buffers: vec![] }
    }
}

/// [UringReader] is designed to facilitate efficient, asynchronous file I/O operations using
/// io_uring, a high-performance I/O interface for Linux Its primary purpose is to read data from a
/// file asynchronously, minimizing I/O latency and maximizing throughput, especially beneficial in
/// applications requiring high-speed data processing.
pub struct UringReader {
    file: File,
    buffers: BufferStore,
    #[cfg(target_os = "linux")]
    io_uring: Option<IoUring>,
    #[cfg(not(target_os = "linux"))]
    io: Option<()>,
    raw_size: usize,
    header_size: usize,
}

#[cfg(target_os = "linux")]
impl UringReader {
    pub fn new(file: File, raw_size: usize, header_size: usize) -> OperationResult<Self> {
        let buffers = BufferStore::new(DISK_PARALLELISM, raw_size);
        let io_uring = IoUring::new(DISK_PARALLELISM as _)?;

        Ok(Self {
            file,
            buffers,
            io_uring: Some(io_uring),
            raw_size,
            header_size,
        })
    }

    /// Takes in iterator of point offsets, reads it, and yields a callback with the read data.
    pub fn read_stream(
        &mut self,
        points: impl IntoIterator<Item = PointOffsetType>,
        mut callback: impl FnMut(usize, PointOffsetType, &[VectorElementType]),
    ) -> OperationResult<()> {
        // Take `UringReader::io_uring`, so that if we return an error or panic during
        // `read_stream`, `IoUring` would be transparently dropped.
        let mut io_uring = match self.io_uring.take() {
            // Use existing `IoUring` if there's one...
            Some(io_uring) => io_uring,
            // ...or create a new one if not
            None => IoUring::new(DISK_PARALLELISM as _)?,
        };

        let buffers_count = self.buffers.buffers.len();
        let mut unused_buffer_ids = (0..buffers_count).collect::<Vec<_>>();

        for item in points.into_iter().enumerate() {
            let (idx, point): (usize, PointOffsetType) = item;

            if unused_buffer_ids.is_empty() {
                submit_and_read(
                    &mut io_uring,
                    &mut self.buffers,
                    &mut unused_buffer_ids,
                    &mut callback,
                    self.raw_size,
                )?;
            }
            // Assume there is at least one buffer available at this point
            let buffer_id = unused_buffer_ids.pop().unwrap();

            self.buffers.buffers[buffer_id].meta = Some(BufferMeta {
                index: idx,
                point_id: point,
            });

            let buffer = &mut self.buffers.buffers[buffer_id].buffer;
            let offset = self.header_size + self.raw_size * point as usize;

            let user_data = buffer_id;

            let read_e = opcode::Read::new(
                Fd(self.file.as_raw_fd()),
                buffer.as_mut_ptr(),
                buffer.len() as _,
            )
            .offset(offset as _)
            .build()
            .user_data(user_data as _);

            unsafe {
                // self.io_uring.submission().push(&read_e).unwrap();
                io_uring.submission().push(&read_e).map_err(|err| {
                    OperationError::service_error(format!("Failed using io-uring: {}", err))
                })?;
            }
        }

        let mut operations_to_wait_for = self.buffers.buffers.len() - unused_buffer_ids.len();

        while operations_to_wait_for > 0 {
            submit_and_read(
                &mut io_uring,
                &mut self.buffers,
                &mut unused_buffer_ids,
                &mut callback,
                self.raw_size,
            )?;

            operations_to_wait_for = self.buffers.buffers.len() - unused_buffer_ids.len();
        }

        self.io_uring = Some(io_uring);

        Ok(())
    }
}

fn submit_and_read(
    io_uring: &mut IoUring,
    buffers: &mut BufferStore,
    unused_buffer_ids: &mut Vec<usize>,
    mut callback: impl FnMut(usize, PointOffsetType, &[VectorElementType]),
    raw_size: usize,
) -> OperationResult<()> {
    let buffers_count = buffers.buffers.len();
    let used_buffers_count = buffers_count - unused_buffer_ids.len();

    // Wait for at least one buffer to become available
    io_uring.submit_and_wait(used_buffers_count)?;

    let cqe = io_uring.completion();
    for entry in cqe {
        let result = entry.result();
        if result < 0 {
            return Err(OperationError::service_error(format!(
                "io_uring operation failed with {} error",
                result
            )));
        } else if (result as usize) != raw_size {
            return Err(OperationError::service_error(format!(
                "io_uring operation returned {} bytes instead of {}",
                result, raw_size
            )));
        }

        let buffer_id = entry.user_data() as usize;
        let meta = buffers.buffers[buffer_id].meta.take().unwrap();
        let buffer = &buffers.buffers[buffer_id].buffer;
        let vector = transmute_from_u8_to_slice(buffer);
        callback(meta.index, meta.point_id, vector);
        unused_buffer_ids.push(buffer_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io,
        io::{Seek, Write},
    };

    use tempfile::tempfile;

    use crate::engine::storage::vector::async_io::UringReader;

    // Helper function to create a temporary file with test data
    fn create_test_file(data: &[u8]) -> io::Result<File> {
        let mut file = tempfile()?;
        file.write_all(data)?;
        file.sync_all()?;
        file.seek(io::SeekFrom::Start(0))?;
        Ok(file)
    }

    #[test]
    fn test_successful_read() {
        let header = vec![0; 128];
        let points_data = vec![1, 2, 3, 4];
        let file_data = [header.clone(), points_data.clone()].concat();
        let file = create_test_file(&file_data).expect("Failed to create test file");

        let raw_size = points_data.len();
        let header_size = header.len();

        let mut reader =
            UringReader::new(file, raw_size, header_size).expect("Failed to create UringReader");
        // Define a point offset to read from (after the header)
        let point_offsets = vec![0];

        // Collect the results in a vector for assertion
        let mut results = Vec::new();

        reader
            .read_stream(point_offsets, |idx, point_id, data| {
                results.push((idx, point_id, data.to_vec()));
            })
            .expect("Failed to read stream");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0); // index
        assert_eq!(results[0].1, 0); // point_id

        let result_data = &results[0].2;
        for data in result_data {
            let data_pts = data.to_ne_bytes().to_vec();
            assert_eq!(data_pts, points_data);
        }
    }

    #[test]
    fn test_read_with_invalid_offset() {
        let header = vec![0; 128];
        let points_data = vec![1, 2, 3, 4];
        let file_data = [header.clone(), points_data].concat();
        let file = create_test_file(&file_data).expect("Failed to create test file");

        let raw_size = 4;
        let header_size = header.len();

        let mut reader =
            UringReader::new(file, raw_size, header_size).expect("Failed to create UringReader");

        // Define an invalid point offset that is beyond the file size
        let point_offsets = vec![1000];

        // Attempt to read from the invalid offset and expect an error
        let result = reader.read_stream(point_offsets, |_, _, _| {});
        assert!(result.is_err());
    }
}
