// Run
// cargo run --no-default-features --features server
// CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner RUSTFLAGS=--cfg=web_sys_unstable_apis cargo run --target wasm32-unknown-unknown --no-default-features --features client

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_particle_systems::ParticleSystemPlugin;

mod helper;
// mod replicon_components;

mod map;
mod network;
mod player;

// use network::*;

use map::MapPlugin;
use network::NetworkPlugin;
use player::{Player, PlayerPlugin};
// use replicon_components::RepliconComponentsPlugin;

// use bevy_replicon::{
//     prelude::*,
//     renet::{
//         transport::{
//             ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
//             ServerAuthentication, ServerConfig,
//         },
//         ClientId, ConnectionConfig, ServerEvent,
//     },
// };
use bevy_xpbd_2d::prelude::*;

// #[derive(Parser, PartialEq, Resource)]
// enum Cli {
//     Server {
//         #[arg(short, long, default_value_t = PORT)]
//         port: u16,
//     },
//     Client {
//         #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
//         ip: IpAddr,

//         #[	arg(short, long, default_value_t = PORT)]
//         port: u16,
//     },
// }

// impl Default for Cli {
//     fn default() -> Self {
//         Self::parse()
//     }
// }

fn main() {
	App::new()
		// .init_resource::<Cli>() // Parse CLI before creating window.
		.add_plugins((
			DefaultPlugins,
			ParticleSystemPlugin::default(),
			PhysicsPlugins::default(),
			NetworkPlugin,
		))
		// .add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, setup)
		.add_systems(
			PostUpdate,
			update_camera
				.after(PhysicsSet::Sync)
				.before(TransformSystem::TransformPropagate),
		)
		.add_plugins((MapPlugin, PlayerPlugin))
		// .add_plugins((
		// 	LogDiagnosticsPlugin::default(),
		// 	FrameTimeDiagnosticsPlugin::default(),
		// ))
		// .insert_resource(Time::<Fixed>::from_seconds(1. / 60.))
		// .add_systems(Update, server_event_system)
		.run();
}

#[derive(Resource)]
struct ClientIdResource(pub u128);

#[derive(Bundle)]
struct PlayerCameraBundle {
	player_camera: PlayerCamera,
	camera: Camera2dBundle,
}

#[derive(Component)]
pub struct PlayerCamera;

fn setup(mut commands: Commands) {
	commands.spawn(PlayerCameraBundle {
		player_camera: PlayerCamera,
		camera: Camera2dBundle {
			transform: Transform::from_xyz(0., 0., 1.),
			projection: OrthographicProjection {
				scale: 6.5,
				..default()
			},
			..default()
		},
	});
}

// fn update_camera(
// 	player_position_query: Query<&Position, With<Player>>,
// 	mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
// 	time: Res<Time>,
// ) {
// 	let player_position = player_position_query.get_single().unwrap().extend(1.);
// 	let mut transform = camera_query.get_single_mut().unwrap();

// 	transform.translation = transform
// 		.translation
// 		.lerp(player_position.as_vec3(), time.delta_seconds() * 10.);
// }

fn update_camera(
	player_position_query: Query<(&Position, &Player)>,
	mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
	time: Res<Time>,
	// client_id: Res<ClientIdResource>,
) {
	for (player_position, _player) in &player_position_query {
		let mut transform = camera_query.get_single_mut().unwrap();

		transform.translation = transform.translation.lerp(
			player_position.extend(1.).as_vec3(),
			time.delta_seconds() * 10.,
		);
	}
}
