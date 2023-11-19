use bevy::{math::DVec2, prelude::*, render::mesh::VertexAttributeValues};
use bevy_particle_systems::*;
use bevy_xpbd_2d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::map::AffectedByGravity;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_player)
			.add_systems(FixedUpdate, apply_player_movement);
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
	gravity_affected: AffectedByGravity,
	player: Player,
	mesh: ColorMesh2dBundle,
	collider: Collider,
	locked_axes: LockedAxes,
	gravity: GravityScale,
	physics: Physics,
}

#[derive(Component, Serialize, Deserialize)]
pub struct Player;

fn setup_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let player_mesh: Mesh = shape::RegularPolygon::new(50., 3).into();

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

	commands
		.spawn(PlayerBundle {
			gravity_affected: AffectedByGravity,
			player: Player,
			mesh: ColorMesh2dBundle {
				mesh: meshes.add(player_mesh).into(),
				material: materials.add(ColorMaterial::from(Color::WHITE)),
				..default()
			},

			collider: Collider::triangle(
				player_mesh_positions[0],
				player_mesh_positions[1],
				player_mesh_positions[2],
			),
			locked_axes: LockedAxes::new(),
			gravity: GravityScale(0.),

			physics: Physics {
				pos: Position(DVec2::new(1200., 0.)),
				l_vel: LinearVelocity(DVec2::new(0., 1000.)),
				m: Mass(1.),
				ext_f: ExternalForce::new(DVec2::ZERO).with_persistence(false),

				..Default::default()
			},
		})
		.with_children(|parent| {
			parent
				.spawn(ParticleSystemBundle {
					particle_system: ParticleSystem {
						max_particles: 20_000,
						texture: ParticleTexture::Sprite(asset_server.load("px.png")),
						spawn_rate_per_second: 0.0.into(),
						initial_speed: JitteredValue::jittered(1000.0, -200.00..200.00),
						lifetime: JitteredValue::jittered(1.0, -0.5..0.5),
						color: ColorOverTime::Gradient(Curve::new(vec![
							CurvePoint::new(Color::rgba(1., 1., 1., 1.), 0.0),
							CurvePoint::new(Color::rgba(1., 1., 0., 1.), 0.1),
							CurvePoint::new(Color::rgba(1., 0., 0., 1.), 0.4),
							CurvePoint::new(Color::rgba(0., 0., 1., 0.), 1.0),
						])),

						looping: true,
						system_duration_seconds: 10.0,

						rescale_texture: None,
						emitter_shape: EmitterShape::CircleSegment(CircleSegment {
							opening_angle: 0.6 * std::f32::consts::PI,
							direction_angle: 3. * std::f32::consts::PI / 2.,
							radius: JitteredValue::jittered(30., 0.0..60.0),
						}),
						velocity_modifiers: vec![],
						scale: ValueOverTime::Curve(Curve::new(vec![
							CurvePoint::new(0.1 * 50., 0.0),
							CurvePoint::new(0.5 * 50., 0.5),
							CurvePoint::new(0.08 * 50., 0.7),
							CurvePoint::new(0.0 * 50., 1.0),
						])),
						initial_rotation: 0.0.into(),
						rotation_speed: 0.0.into(),
						rotate_to_movement_direction: false,
						max_distance: None,
						z_value_override: Some(JitteredValue {
							value: 0.1,
							jitter_range: None,
						}),
						bursts: Vec::default(),
						space: ParticleSpace::World,
						use_scaled_time: true,
						despawn_on_finish: false,
						despawn_particles_with_system: false,
					},
					..ParticleSystemBundle::default()
				})
				.insert(Playing);
		});
}

fn apply_player_movement(
	time_step: Res<Time<Fixed>>,
	keys: Res<Input<KeyCode>>,
	mut player_query: Query<(
		(
			&mut ExternalForce,
			&mut AngularVelocity,
			&LinearVelocity,
			&Rotation,
			&Children,
		),
		With<Player>,
	)>,
	mut particle_effect_query: Query<(&mut Transform, &mut ParticleSystem)>,
) {
	const THRUST_PARTICLE_SPAWN_RATE: f32 = 500.0;
	const THRUST_PARTICLE_VELOCITY: f64 = 200.0;

	let ((mut ext_forces, mut avel, lvel, rot, children), _) =
		player_query.get_single_mut().unwrap();

	let child_id = *children.get(0).unwrap(); // Thruster ID BC only 1 child that is the truster

	if keys.pressed(KeyCode::W) {
		// dbg!(&ext_forces);
		ext_forces.apply_force(rot.rotate(DVec2::Y * 1.5e8 * time_step.timestep().as_secs_f64()));

		particle_effect_query
			.get_mut(child_id)
			.unwrap()
			.1
			.spawn_rate_per_second = THRUST_PARTICLE_SPAWN_RATE.into();
	} else {
		particle_effect_query
			.get_mut(child_id)
			.unwrap()
			.1
			.spawn_rate_per_second = 0.0.into();
	}

	let rot = Rotation::from_degrees(rot.as_degrees() - 90.);
	let particle_velocity: DVec2 =
		lvel.0 + (DVec2::new(rot.cos(), rot.sin()) * THRUST_PARTICLE_VELOCITY);

	particle_effect_query
		.get_mut(child_id)
		.unwrap()
		.1
		.initial_speed = JitteredValue {
		value: ((lvel.0.length() + THRUST_PARTICLE_VELOCITY) as f32),
		jitter_range: Some(-300.0..300.0),
	}; // (particle_velocity.length().abs() as f32).into();
	particle_effect_query
		.get_mut(child_id)
		.unwrap()
		.1
		.initial_rotation = (particle_velocity.angle_between(DVec2::new(1., 0.)) as f32).into();

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

	avel.0 *= 1. - ((1. - 0.2) * time_step.timestep().as_secs_f64());
}
