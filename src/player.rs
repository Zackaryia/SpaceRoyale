use core::fmt;

use bevy::{math::{DVec2, DVec3}, prelude::*, render::mesh::VertexAttributeValues};
use bevy_particle_systems::*;
use bevy_simplenet::{Client, Server};
// use bevy_replicon::renet::ClientId;
use bevy_xpbd_2d::prelude::*;
use serde::{Deserialize, Serialize, Serializer, Deserializer};

// use bevy_replicon::prelude::*;

use crate::{map::AffectedByGravity, network::{ClientMsgEvent, NetworkChannel}, ClientMsg};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			// .replicate::<Player>()
			.add_event::<Inputs>()
			.add_systems(Update, 
				(apply_player_movement.run_if(resource_exists::<Server<NetworkChannel>>()), // Runs only on the server or a single player.
				input_system.run_if(resource_exists::<Client<NetworkChannel>>()))
			)
			// .add_systems(Startup, spawn_player)
			.add_systems(PreUpdate, player_init_system);
	}
}

#[derive(Bundle)]struct Physics {
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
	// replication: Replication,

	gravity_affected: AffectedByGravity,
	mesh: ColorMesh2dBundle,
	collider: Collider,
	locked_axes: LockedAxes,
	gravity: GravityScale,
	physics: Physics,
}

#[derive(Component, Clone, Copy)]
pub struct Player;

// impl Serialize for Player {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_u64(self.0.raw())
//     }
// }

// use serde::de::{self, Visitor};

// struct PlayerVisitor;

// impl<'de> Visitor<'de> for PlayerVisitor {
//     type Value = Player;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("Bruh help")
//     }

//     fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
//     where
//         E: de::Error,
//     {
//         Ok(Player)
//     }
// }

// impl<'de> Deserialize<'de> for Player {
//     fn deserialize<D>(deserializer: D) -> Result<Player, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_u64(PlayerVisitor)
//     }
// }

fn player_init_system(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	asset_server: Res<AssetServer>,
	spawned_players: Query<(&Player, Entity), Added<Player>>,
) {
	for (player, entity) in &spawned_players {
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

		commands.get_entity(entity).unwrap().log_components();

		commands.entity(entity).insert(PlayerBundle {
			player: *player,
			// replication: Replication,
			gravity_affected: AffectedByGravity,
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
				m: Mass(1.),
				ext_f: ExternalForce::new(DVec2::ZERO).with_persistence(false),
				..Default::default()
			},
		});

		commands.entity(entity).with_children(|parent| {
			parent.spawn(ParticleSystemBundle {
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
}

/// A movement event for the controlled box.
#[derive(Debug, Default, Deserialize, Event, Serialize, Clone)]
pub struct Inputs {
	w: bool,
	a: bool,
	d: bool,
}

fn input_system(
	mut move_events: EventWriter<ClientMsgEvent>,
	client_network: Res<Client<NetworkChannel>>,
	keys: Res<Input<KeyCode>>,
) {
	move_events.send(ClientMsgEvent { msg: crate::ClientMsg::Input(Inputs {
		w: keys.pressed(KeyCode::W),
		a: keys.pressed(KeyCode::A),
		d: keys.pressed(KeyCode::D),
	}), id: client_network.id() });
}


fn apply_player_movement(
	time_step: Res<Time>,
	mut move_events: EventReader<ClientMsgEvent>,
	mut player_query: Query<(
		&Player,
		&mut ExternalForce,
		&mut AngularVelocity,
		&LinearVelocity,
		&Rotation,
		&Children,
	)>,
	mut particle_effect_query: Query<(&mut Transform, &mut ParticleSystem)>,
) {
	const THRUST_PARTICLE_SPAWN_RATE: f32 = 500.0;
	const THRUST_PARTICLE_VELOCITY: f64 = 200.0;

	for input in move_events.read() {
		let ClientMsgEvent { msg: ClientMsg::Input(input), .. } = input.clone() else {
			continue;
		};

		// info!("received event {event:?} from client {client_id}");
		for (player, mut ext_forces, mut avel, lvel, rot, children) in &mut player_query {
			let child_id = *children.get(0).unwrap(); // Thruster ID BC only 1 child that is the truster

			if input.w {
				// dbg!(&ext_forces);
				ext_forces.apply_force(rot.rotate(DVec2::Y * 1.5e8 * time_step.delta().as_secs_f64()));
			
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
			
			if input.a {
				avel_change += 6.;
			}
			
			if input.d {
				avel_change -= 6.;
			}
			
			if avel_change != 0. {
				avel.0 += avel_change * time_step.delta().as_secs_f64();
			}
			
			avel.0 *= 1. - ((1. - 0.2) * time_step.delta().as_secs_f64());
		}
	}
}
