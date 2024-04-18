use std::{io, path::PathBuf};

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum MMapSyncError {
    #[snafu(display("Unable to read state file from {}", path.display()))]
    OpenStateFile { source: io::Error, path: PathBuf },
    #[snafu(display("Unable to read state file metadata from {}", path.display()))]
    StateFileMeta { source: io::Error, path: PathBuf },
    #[snafu(display("Set state file len error to {}, len: {}", path.display(), len))]
    SetStateFileLen {
        source: io::Error,
        path: PathBuf,
        len: usize,
    },
    #[snafu(display("Unable to read data file from {}", path.display()))]
    DataFileRead { source: io::Error, path: PathBuf },
    #[snafu(display("Unable to write data file from {}", path.display()))]
    DataFileWrite { source: io::Error, path: PathBuf },
    #[snafu(display("error reading entity"))]
    ReadEntityData { source: io::Error },
    #[snafu(display(
        "error reading entity, data size {} is lower than current data size {}",
        data_file_size,
        current_size
    ))]
    DataVersionMiss {
        data_file_size: usize,
        current_size: usize,
    },
    #[snafu(display("invalid version param, idx: {}, size: {}", idx, size))]
    InvalidVersion { idx: usize, size: usize },
    #[snafu(display("uninitialized state"))]
    UninitializedState,
}
