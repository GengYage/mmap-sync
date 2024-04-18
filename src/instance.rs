use std::ffi::{OsStr, OsString};

use crate::error::MMapSyncError;

const DATA_SIZE_BITS: usize = 39;
const DATA_CHECKSUM_BITS: usize = 24;

/// `InstanceVersion` represents data instance and consists of the following components:
/// - data idx (0 or 1)   - 1 bit
/// - data size (<549 GB) - 39 bits
/// - data checksum       - 24 bits
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstanceVersion(pub u64);

impl InstanceVersion {
    /// Create new `InstanceVersion` from data instance `idx`, `size` and `checksum`
    #[inline]
    pub fn new(idx: usize, size: usize, checksum: u64) -> Result<InstanceVersion, MMapSyncError> {
        let mut res: u64 = 0;

        if idx > 1 || size >= 1 << DATA_SIZE_BITS {
            return Err(MMapSyncError::InvalidVersion { idx, size });
        }

        res |= (idx as u64) & 1;
        res |= ((size as u64) & ((1 << DATA_SIZE_BITS) - 1)) << 1;
        res |= (checksum & ((1 << DATA_CHECKSUM_BITS) - 1)) << (DATA_SIZE_BITS + 1);

        Ok(InstanceVersion(res))
    }

    /// Get data instance `idx` (0 or 1)
    #[inline]
    pub fn idx(&self) -> usize {
        self.0 as usize & 1
    }

    /// Get data instance `size`
    #[inline]
    pub fn size(&self) -> usize {
        (self.0 as usize >> 1) & ((1 << DATA_SIZE_BITS) - 1)
    }

    /// Get data instance `checksum`
    #[cfg(test)]
    pub fn checksum(&self) -> u64 {
        self.0 >> (DATA_SIZE_BITS + 1)
    }

    /// Get data instance `path`
    #[inline]
    pub fn path(&self, path_prefix: &OsStr) -> OsString {
        let mut path = path_prefix.to_os_string();
        path.push(format!("_data_{}", self.idx()));
        path
    }
}

impl TryFrom<u64> for InstanceVersion {
    type Error = MMapSyncError;

    #[inline]
    fn try_from(v: u64) -> Result<InstanceVersion, Self::Error> {
        if v == 0 {
            Err(MMapSyncError::UninitializedState)
        } else {
            Ok(InstanceVersion(v))
        }
    }
}

impl From<InstanceVersion> for u64 {
    #[inline]
    fn from(v: InstanceVersion) -> Self {
        v.0
    }
}
