use std::{fmt, fs};
use std::path::{Path, PathBuf};

use crate::common::operation_error::{OperationError, OperationResult};

fn assert_is_dir(dir: &Path) -> OperationResult<()> {
  if dir.is_dir() {
    Ok(())
  } else {
    Err(not_a_dir_error(dir))
  }
}

fn not_a_dir_error(dir: &Path) -> OperationError {
  OperationError::service_error(format!(
    "path {dir:?} is not a directory (or does not exist)"
  ))
}

fn failed_to_read_dir_error(dir: &Path, err: impl fmt::Display) -> OperationError {
  OperationError::service_error(format!("failed to read {dir:?} directory: {err}"))
}

fn failed_to_move_error(path: &Path, dest: &Path, err: impl fmt::Display) -> OperationError {
  OperationError::service_error(format!("failed to move {path:?} to {dest:?}: {err}"))
}

/// Finds the first symlink in the directory tree and returns its path.
pub fn find_symlink(directory: &Path) -> Option<PathBuf> {
  let entries = match fs::read_dir(directory) {
    Ok(entries) => entries,
    Err(_) => return None,
  };

  for entry in entries {
    let entry = match entry {
      Ok(entry) => entry,
      Err(_) => continue,
    };

    let path = entry.path();

    if path.is_dir() {
      if let Some(path) = find_symlink(&path) {
        return Some(path);
      }
    } else if path.is_symlink() {
      return Some(path);
    }
  }

  None
}
