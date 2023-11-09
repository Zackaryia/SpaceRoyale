use std::default;

use bevy::{math::DVec2, prelude::*, render::mesh::VertexAttributeValues};
use bevy_xpbd_2d::{prelude::*, math::PI};

use crate::{planet::Planet, thrust};

const GRAVITATIONAL_CONSTANT: f64 = 7e-11 * 1e7;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(FixedUpdate, (apply_gravity, apply_player_movement));
    }
}

#[derive(Bundle)]
struct Physics {
    rbody: RigidBody,
    pos: Position,
    rot: Rotation,
    l_vel: LinearVelocity,
    a_vel: AngularVelocity,
    ext_f: ExternalForce,
    ext_t: ExternalTorque,
    ext_i: ExternalImpulse,
    ext_ai: ExternalAngularImpulse,
    f: Friction,
    r: Restitution,
    m: Mass,
    i: Inertia,
    com: CenterOfMass,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            rbody: RigidBody::Dynamic,
            pos: Position::default(),
            rot: Rotation::default(),
            l_vel: LinearVelocity::default(),
            a_vel: AngularVelocity::default(),
            ext_f: ExternalForce::default(),
            ext_t: ExternalTorque::default(),
            ext_i: ExternalImpulse::default(),
            ext_ai: ExternalAngularImpulse::default(),
            f: Friction::default(),
            r: Restitution::default(),
            m: Mass::default(),
            i: Inertia::default(),
            com: CenterOfMass::default(),
        }
    }
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    mesh: ColorMesh2dBundle,
    collider: Collider,
    locked_axes: LockedAxes,
    gravity: GravityScale,
    physics: Physics,
}

#[derive(Component)]
pub struct Player;

/// A simple marker component to identify the effect using a dynamic
/// property-based acceleration that the `update_accel()` system will control at
/// runtime.
#[derive(Component)]
struct DynamicRuntimeAccel;



use bevy_hanabi::prelude::*;



fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let player_mesh: Mesh = shape::RegularPolygon::new(30., 3).into();

    let mut player_mesh_positions = Vec::new();
    if let VertexAttributeValues::Float32x3(data) =
        player_mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
    {
        player_mesh_positions = data
            .clone()
            .iter()
            .map(|x| DVec2::from([x[0] as f64, x[1] as f64]))
            .collect();
    }
    assert!(player_mesh_positions.len() == 3);

    let spawner = Spawner::rate(100_f32.into());

    commands.spawn(PlayerBundle {
        player: Player,
        mesh: ColorMesh2dBundle {
            mesh: meshes.add(player_mesh).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        },

        collider: Collider::triangle(player_mesh_positions[0], player_mesh_positions[1], player_mesh_positions[2]),
        locked_axes: LockedAxes::new(),
        gravity: GravityScale(0.),

        physics: Physics {
            pos: Position(DVec2::new(1200., 0.)),
            l_vel: LinearVelocity(DVec2::new(0., 620.)),
            m: Mass(1.),
            ext_f: ExternalForce::new(DVec2::ZERO).with_persistence(false),

            ..Default::default()
        },
    }).with_children(|parent| {
        parent.spawn(
            ParticleEffectBundle {
                effect: ParticleEffect::new(thrust::setup_thrust_particles(effects)).with_spawner(spawner),
                transform: Transform::from_translation(Vec3::Y).with_rotation(Quat::from_rotation_arc_2d(Vec2::X, Vec2::NEG_Y)),
                ..Default::default()
            },
        );
    });
}

fn apply_gravity(
    time_step: Res<Time<Fixed>>,
    mut player_query: Query<(&Player, &Position, &Mass, &mut ExternalForce)>,
    planet_query: Query<(&Planet, &Position, &Mass)>,
) {
    let (_, player_position, player_mass, mut external_force) =
        player_query.get_single_mut().unwrap();
    let (_, planet_position, planet_mass) = planet_query.get_single().unwrap();

    let grav_direction = planet_position.0 - player_position.0;

    let force = time_step.timestep().as_secs_f64()
        * GRAVITATIONAL_CONSTANT
        * ((player_mass.0 * planet_mass.0) / grav_direction.length_squared());

    let direction_norm = grav_direction.normalize();
    let force_vec = direction_norm * force;

    external_force.apply_force(force_vec);
}

fn apply_player_movement(
	time_step: Res<Time<Fixed>>,
	keys: Res<Input<KeyCode>>,
    mut player_query: Query<((&mut ExternalForce, &mut AngularVelocity, &Rotation), With<Player>)>,
) {
	let ((mut ext_forces, mut avel, rot), _) = player_query.get_single_mut().unwrap();

	if keys.pressed(KeyCode::W) {
		dbg!(&ext_forces);
		ext_forces.apply_force(rot.rotate(DVec2::Y * 10000000. * time_step.timestep().as_secs_f64())); 

	}

	let mut avel_change = 0.;

	if keys.pressed(KeyCode::A) {
		avel_change += 6.;
	}

	if keys.pressed(KeyCode::D) {
		avel_change -= 6.;
	}

	
	if avel_change != 0. {
		avel.0 += avel_change * time_step.timestep().as_secs_f64();
	}

    avel.0 *= 1. - ((1. - 0.1) * time_step.timestep().as_secs_f64());
}
