use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to copy {0} to {1}: {2}")]
    CopyFileFailed(PathBuf, PathBuf, std::io::Error),

    #[error("Failed to create {0}: {1}")]
    CreateDirectoryFailed(PathBuf, std::io::Error),

    #[error("Cannot determine parent folder for {0}")]
    NoParent(PathBuf),

    #[error("Failed to read {0}: {1}")]
    ReadFileFailed(PathBuf, std::io::Error),

    #[error("Failed to read permissions of {0}: {1}")]
    ReadPermissionsFailed(PathBuf, std::io::Error),

    #[error("Failed to remove directory {0}: {1}")]
    RemoveDirectoryFailed(PathBuf, std::io::Error),

    #[error("Failed to remove file {0}: {1}")]
    RemoveFileFailed(PathBuf, std::io::Error),

    #[error("Failed to rename {0} to {1}: {2}")]
    RenameFailed(PathBuf, PathBuf, std::io::Error),

    #[error("Failed to write to {0}: {1}")]
    WriteFileFailed(PathBuf, std::io::Error),

    #[error("Failed to set permissions of {0}: {1}")]
    WritePermissionsFailed(PathBuf, std::io::Error),
}