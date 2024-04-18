use std::ops::Deref;

use crate::{instance::InstanceVersion, state::State};

/// An RAII implementation of a “scoped read lock” of a `State`
pub struct ReadGuard<'a> {
    state: &'a mut State,
    version: InstanceVersion,
}

impl<'a> ReadGuard<'a> {
    pub fn new(state: &'a mut State, version: InstanceVersion) -> Self {
        state.rlock(version);
        Self { state, version }
    }
}

impl<'a> Drop for ReadGuard<'a> {
    fn drop(&mut self) {
        self.state.runlock(self.version);
    }
}

pub struct ReadResult<'a, T: Sized> {
    _guard: ReadGuard<'a>,
    entity: &'a T,
    switched: bool,
}

impl<'a, T: Sized> ReadResult<'a, T> {
    pub fn new(_guard: ReadGuard<'a>, entity: &'a T, switched: bool) -> Self {
        Self {
            _guard,
            entity,
            switched,
        }
    }

    pub fn is_switched(&self) -> bool {
        self.switched
    }
}

impl<'a, T: Sized> Deref for ReadResult<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.entity
    }
}
