use bevy::{prelude::*, math::DVec2};
use bevy_particle_systems::Line;
use bevy_replicon::{prelude::*, bincode, renet::ClientId};
use bevy_xpbd_2d::components::{LinearVelocity, AngularVelocity, ExternalForce, Rotation, Position};
use std::io::Cursor;
use bevy::{prelude::*, ptr::Ptr};
use bevy_replicon::{prelude::*, renet::Bytes, replicon_core::replication_rules};
use serde::{Deserialize, Serialize};
use crate::player::Player;


// app.replicate_with::<Transform>(serialize_transform, deserialize_transform, replication_rules::remove_component::<Transform>)
//     .replicate::<Player>()
//     .add_systems(PreUpdate, player_init_system.after(ClientSet::Receive));

// rbody: RigidBody::Dynamic,
// pos: Position::default(),
// rot: Rotation::default(),
// l_vel: LinearVelocity::default(),
// a_vel: AngularVelocity::default(),
// ext_f: ExternalForce::default(),
// ext_t: ExternalTorque::default(),
// ext_i: ExternalImpulse::default(),
// ext_ai: ExternalAngularImpulse::default(),
// f: Friction::default(),
// r: Restitution::default(),
// m: Mass::default(),
// i: Inertia::default(),
// com: CenterOfMass::default(),

pub struct RepliconComponentsPlugin;

impl Plugin for RepliconComponentsPlugin {
	fn build(&self, app: &mut App) {
        app
        .replicate_with::<Transform>(serialize_transform, deserialize_transform, replication_rules::remove_component::<Transform>)
        // .replicate_with::<Position>(serialize_position, deserialize_position, replication_rules::remove_component::<Position>)
        // .replicate_with::<Rotation>(serialize_rotation, deserialize_rotation, replication_rules::remove_component::<Rotation>)
        .replicate_with::<LinearVelocity>(serialize_linear_velocity, deserialize_linear_velocity, replication_rules::remove_component::<LinearVelocity>)
        .replicate_with::<AngularVelocity>(serialize_angular_velocity, deserialize_angular_velocity, replication_rules::remove_component::<AngularVelocity>)
        .replicate_with::<ExternalForce>(serialize_external_force, deserialize_external_force, replication_rules::remove_component::<ExternalForce>)
        .replicate_with::<Player>(serialize_player, deserialize_player, replication_rules::remove_component::<Player>)
            
            ;
	}
}

fn serialize_position(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let position: &Position = unsafe { component.deref() };
    bincode::serialize_into(cursor, &position.0)
}

fn deserialize_position(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let dvec2: DVec2 = bincode::deserialize_from(cursor)?;
    entity.insert(Position::new(dvec2));

    Ok(())
}

fn serialize_rotation(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let rotation: &Rotation = unsafe { component.deref() };
    bincode::serialize_into(cursor, &rotation.as_radians())
}

fn deserialize_rotation(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let rotation: f64 = bincode::deserialize_from(cursor)?;
    entity.insert(Rotation::from_radians(rotation));

    Ok(())
}


/// Serializes only translation.
fn serialize_transform(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let transform: &Transform = unsafe { component.deref() };
    bincode::serialize_into(cursor, &transform.compute_matrix())
}

/// Deserializes translation and creates [`Transform`] from it.
fn deserialize_transform(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let matrix: Mat4 = bincode::deserialize_from(cursor)?;
    entity.insert(Transform::from_matrix(matrix));

    Ok(())
}


fn serialize_linear_velocity(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let liner_velocity: &LinearVelocity = unsafe { component.deref() };
    bincode::serialize_into(cursor, &liner_velocity.0)
}

fn deserialize_linear_velocity(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let liner_velocity_dvec: DVec2 = bincode::deserialize_from(cursor)?;
    entity.insert(LinearVelocity::from(liner_velocity_dvec));

    Ok(())
}


fn serialize_angular_velocity(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let angular_velocity: &AngularVelocity = unsafe { component.deref() };
    bincode::serialize_into(cursor, &angular_velocity.0)
}

fn deserialize_angular_velocity(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let angular_velocity: f64 = bincode::deserialize_from(cursor)?;
    entity.insert(AngularVelocity::from(angular_velocity));

    Ok(())
}



fn serialize_external_force(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let external_force: &ExternalForce = unsafe { component.deref() };
    bincode::serialize_into(cursor, &(&external_force.force(), &external_force.persistent))//, &external_force.torque()))
}

fn deserialize_external_force(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let (force, persistnat): (DVec2, bool) = bincode::deserialize_from(cursor)?;
    let mut ef = ExternalForce::new(force);
    ef.persistent = persistnat;
    entity.insert(ef);

    Ok(())
}


fn serialize_player(
    component: Ptr,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    // SAFETY: Function called for registered `ComponentId`.
    let player: &Player = unsafe { component.deref() };
    bincode::serialize_into(cursor, &player.0.raw())
}

fn deserialize_player(
    entity: &mut EntityWorldMut,
    _entity_map: &mut ServerEntityMap,
    cursor: &mut Cursor<Bytes>,
    _tick: RepliconTick,
) -> bincode::Result<()> {
    let client_id: u64 = bincode::deserialize_from(cursor)?;
    entity.insert(Player(ClientId::from_raw(client_id)));

    Ok(())
}

