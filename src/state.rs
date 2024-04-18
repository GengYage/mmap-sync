use std::{
    ffi::{OsStr, OsString},
    fs::OpenOptions,
    mem,
    ops::Add,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

use memmap2::MmapMut;
use snafu::ResultExt;

use crate::{
    error::{MMapSyncError, OpenStateFileSnafu, SetStateFileLenSnafu, StateFileMetaSnafu},
    instance::InstanceVersion,
};

const STATE_SUFFIX: &str = "_state";
const STATE_SIZE: usize = mem::size_of::<State>();
const SLEEP_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug)]
#[repr(C)]
pub struct State {
    /// Current data instance version
    version: AtomicU64,
    /// Current number of readers for each data instance
    idx_readers: [AtomicU32; 2],
}

impl State {
    pub fn new() -> State {
        State {
            version: AtomicU64::new(0),
            idx_readers: [AtomicU32::new(0), AtomicU32::new(0)],
        }
    }

    pub fn version(&self) -> Result<InstanceVersion, MMapSyncError> {
        self.version.load(Ordering::SeqCst).try_into()
    }

    pub fn acquire_next_idx(&self, grace_duration: Duration) -> (usize, bool) {
        let next_id = InstanceVersion::try_from(self.version.load(Ordering::SeqCst))
            .map(|version| (version.idx() + 1) % 2)
            .unwrap_or(0);

        // check number of readers using next_idx
        let num_readers = &self.idx_readers[next_id];

        let grace_expiring_at = Instant::now().add(grace_duration);

        let mut reset = false;

        while num_readers.load(Ordering::SeqCst) > 0 {
            if Instant::now().gt(&grace_expiring_at) {
                // over time
                num_readers.store(0, Ordering::SeqCst);
                reset = true;
                break;
            } else {
                thread::sleep(SLEEP_DURATION);
            }
        }

        (next_id, reset)
    }

    /// Switches state to given `version`
    /// new reader will use the new version
    pub fn switch_version(&mut self, version: InstanceVersion) {
        self.version.swap(version.into(), Ordering::SeqCst);
    }

    /// Locks given `version` of the state for reading
    #[inline]
    pub fn rlock(&mut self, version: InstanceVersion) {
        self.idx_readers[version.idx()].fetch_add(1, Ordering::SeqCst);
    }

    /// Unlocks given `version` from reading
    #[inline]
    pub(crate) fn runlock(&mut self, version: InstanceVersion) {
        self.idx_readers[version.idx()].fetch_sub(1, Ordering::SeqCst);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StateContainer {
    /// state file path
    state_path: OsString,
    /// state file memory map
    mmap: Option<MmapMut>,
}

impl StateContainer {
    pub fn new(path_prefix: &OsStr) -> StateContainer {
        let mut state_path = path_prefix.to_os_string();
        state_path.push(STATE_SUFFIX);
        StateContainer {
            state_path,
            mmap: None,
        }
    }

    pub fn state(&mut self, create: bool) -> Result<&mut State, MMapSyncError> {
        if self.mmap.is_none() {
            let mut opts = OpenOptions::new();
            opts.read(true).write(true).create(create);

            let state_file = opts.open(&self.state_path).context(OpenStateFileSnafu {
                path: &self.state_path,
            })?;

            let mut need_init = false;
            if state_file
                .metadata()
                .context(StateFileMetaSnafu {
                    path: &self.state_path,
                })?
                .len() as usize
                != STATE_SIZE
            {
                state_file
                    .set_len(STATE_SIZE as u64)
                    .context(SetStateFileLenSnafu {
                        path: &self.state_path,
                        len: STATE_SIZE,
                    })?;
                need_init = true;
            }

            let mut mmap = unsafe {
                MmapMut::map_mut(&state_file).context(OpenStateFileSnafu {
                    path: &self.state_path,
                })?
            };

            if need_init {
                let new_state = State::default();
                unsafe {
                    mmap.as_mut_ptr()
                        .copy_from(&new_state as *const State as *const u8, STATE_SIZE)
                }
            }

            self.mmap = Some(mmap);
        }

        Ok(unsafe { &mut *(self.mmap.as_ref().unwrap().as_ptr() as *mut State) })
    }
}
