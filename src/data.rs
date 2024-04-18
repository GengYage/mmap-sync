use std::{
    ffi::{OsStr, OsString},
    fs::{File, OpenOptions},
};

use memmap2::{Mmap, MmapMut};
use snafu::ResultExt;

use crate::{
    error::{DataFileReadSnafu, DataFileWriteSnafu, MMapSyncError},
    instance::InstanceVersion,
};

pub struct DataContainer {
    /// Base data path
    path_prefix: OsString,
    /// Reader's current local instance version
    version: Option<InstanceVersion>,
    /// Read-only memory mapped files storing data
    idx_mmaps: [Option<Mmap>; 2],
}

impl DataContainer {
    pub(crate) fn new(path_prefix: &OsStr) -> Self {
        DataContainer {
            path_prefix: path_prefix.into(),
            version: None,
            idx_mmaps: [None, None],
        }
    }

    pub fn data(&mut self, version: InstanceVersion) -> Result<(&[u8], bool), MMapSyncError> {
        let mmap = &mut self.idx_mmaps[version.idx()];
        let data_size = version.size();

        // only open and mmap data file in the following cases:
        // * if it never was opened/mapped before
        // * if current mmap size is smaller than requested data size
        if mmap.is_none() || mmap.as_ref().unwrap().len() < data_size {
            let path = version.path(&self.path_prefix);

            let data_file = File::open(&path).context(DataFileReadSnafu { path: path.clone() })?;
            let data_file_meta = data_file
                .metadata()
                .context(DataFileReadSnafu { path: path.clone() })?;

            if data_file_meta.len() < data_size as u64 {
                return Err(MMapSyncError::DataVersionMiss {
                    data_file_size: data_file_meta.len() as _,
                    current_size: data_size,
                });
            }

            *mmap = Some(unsafe { Mmap::map(&data_file).context(DataFileReadSnafu { path })? });
        }

        let data = &mmap.as_ref().unwrap()[..data_size];
        let new_version = Some(version);
        let switched = new_version != self.version;
        self.version = new_version;

        Ok((data, switched))
    }

    pub fn write(&mut self, data: &[u8], version: InstanceVersion) -> Result<usize, MMapSyncError> {
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true);

        let data_path = version.path(&self.path_prefix);
        let data_file = opts.open(&data_path).context(DataFileWriteSnafu {
            path: data_path.clone(),
        })?;

        let data_len = data.len() as u64;

        if data_file
            .metadata()
            .context(DataFileWriteSnafu {
                path: data_path.clone(),
            })?
            .len()
            < data_len
        {
            data_file
                .set_len(data_len)
                .context(DataFileWriteSnafu { path: data_path.clone() })?;
        }

        let mut mmap = unsafe {
            MmapMut::map_mut(&data_file).context(DataFileWriteSnafu {
                path: data_path.clone(),
            })?
        };

        // write data to mmap
        mmap[..data.len()].copy_from_slice(data);
        mmap.flush()
            .context(DataFileWriteSnafu { path: data_path })?;

        Ok(data.len())
    }
}
