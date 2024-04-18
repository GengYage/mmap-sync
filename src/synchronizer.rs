use std::{
    ffi::OsStr,
    hash::{DefaultHasher, Hasher},
    mem, slice,
    time::Duration,
};

use crate::{
    data::DataContainer,
    error::MMapSyncError,
    guard::{ReadGuard, ReadResult},
    instance::InstanceVersion,
    state::StateContainer,
};

/// Synchronizer
pub struct Synchronizer {
    /// Container storing state mmap
    state_container: StateContainer,
    /// Container storing data mmap
    data_container: DataContainer,
}

impl Synchronizer {
    /// Create a new Synchronizer
    pub fn new(path_prefix: &OsStr) -> Synchronizer {
        Synchronizer {
            state_container: StateContainer::new(path_prefix),
            data_container: DataContainer::new(path_prefix),
        }
    }

    pub fn read<T: Sized>(&mut self) -> Result<ReadResult<T>, MMapSyncError> {
        let state = self.state_container.state(false)?;

        let version = state.version()?;

        let guard = ReadGuard::new(state, version);

        let (data, switched) = self.data_container.data(version)?;

        let entity = unsafe { &*(data.as_ptr() as *const T) };

        Ok(ReadResult::new(guard, entity, switched))
    }

    pub fn write<T: Sized>(
        &mut self,
        entity: &T,
        grace_duration: Duration,
    ) -> Result<(usize, bool), MMapSyncError> {
        let data = entity as *const T as *const u8;
        let data = unsafe { slice::from_raw_parts(data, mem::size_of::<T>()) };

        let mut hasher = DefaultHasher::new();
        hasher.write(data);
        let checksum = hasher.finish();

        let state = self.state_container.state(true)?;

        let (new_idx, reset) = state.acquire_next_idx(grace_duration);
        let new_version = InstanceVersion::new(new_idx, data.len(), checksum)?;
        let size = self.data_container.write(data, new_version)?;

        // make sure new readers to new version
        state.switch_version(new_version);

        Ok((size, reset))
    }
}
