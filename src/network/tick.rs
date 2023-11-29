use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marks entity for replication.
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Replication;

/// A tick that increments each time we need the server to compute and send an update.
///
/// Used as resource only on server.
/// Mapped to the Bevy's `Tick` in [`AckedTicks`](crate::server::AckedTicks).
/// See also [`TickPolicy`](crate::server::TickPolicy).
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Resource, Serialize)]
pub struct RepliconTick(pub(crate) u32);

#[allow(unused)]
impl RepliconTick {
    /// Gets the value of this tick.
    #[inline]
    pub fn get(self) -> u32 {
        self.0
    }

    /// Increments current tick by the specified `value` and takes wrapping into account.
    #[inline]
    pub fn increment_by(&mut self, value: u32) {
        self.0 = self.0.wrapping_add(value);
    }

    /// Same as [`Self::increment_by`], but increments only by 1.
    #[inline]
    pub fn increment(&mut self) {
        self.increment_by(1)
    }
}

impl PartialOrd for RepliconTick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let difference = self.0.wrapping_sub(other.0);
        if difference == 0 {
            Some(Ordering::Equal)
        } else if difference > u32::MAX / 2 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}


/// Last received tick from server.
///
/// Used only on clients, sent to the server in last replicon ack message.
#[derive(Debug, Default, Deref, Resource)]
pub struct LastRepliconTick(pub(super) RepliconTick);
/// Contains the lowest replicon tick that should be acknowledged by clients.
///
/// If a client has not acked this tick, then replication messages >= this tick
/// will be sent even if they do not contain data.
///
/// Used to synchronize server-sent events with clients. A client cannot consume
/// a server-sent event until it has acknowledged the tick where that event was
/// created. This means we need to replicate ticks after a server-sent event is
/// emitted to guarantee the client can eventually consume the event.
#[derive(Clone, Copy, Debug, Default, Deref, DerefMut, Resource)]
pub(super) struct MinRepliconTick(RepliconTick);
