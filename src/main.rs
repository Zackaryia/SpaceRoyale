use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_particle_systems::ParticleSystemPlugin;


mod helper;
mod replicon_components;

mod map;
mod player;

use map::MapPlugin;
use player::{Player, PlayerPlugin};
use replicon_components::RepliconComponentsPlugin;


use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use clap::Parser;
use serde::{Deserialize, Serialize};

use bevy_replicon::{
    prelude::*,
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        ClientId, ConnectionConfig, ServerEvent,
    },
};
use bevy_xpbd_2d::prelude::*;

const PORT: u16 = 5000;
const PROTOCOL_ID: u64 = 0;

#[derive(Parser, PartialEq, Resource)]
enum Cli {
    Server {
        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
    Client {
        #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
        ip: IpAddr,

        #[	arg(short, long, default_value_t = PORT)]
        port: u16,
    },
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}


fn main() {
	App::new()
		.init_resource::<Cli>() // Parse CLI before creating window.
		.add_plugins((
			DefaultPlugins,
			ReplicationPlugins.build().set(ServerPlugin::new(TickPolicy::MaxTickRate(60))),
			ParticleSystemPlugin::default(),
			PhysicsPlugins::default(), 
			RepliconComponentsPlugin,
		))
		// .add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, (setup, cli_system.map(Result::unwrap)))
		.add_systems(
			PostUpdate,
			update_camera.run_if(not(has_authority()))
				.after(PhysicsSet::Sync)
				.before(TransformSystem::TransformPropagate),
		)
		.add_plugins((MapPlugin, PlayerPlugin))
		// .add_plugins((
		// 	LogDiagnosticsPlugin::default(),
		// 	FrameTimeDiagnosticsPlugin::default(),
		// ))
		.insert_resource(Time::<Fixed>::from_seconds(1. / 60.))
		.add_systems(Update, server_event_system.run_if(has_authority()))
		.run();
}

#[derive(Resource)]
struct ClientIdResource(pub u64);

fn cli_system(
	mut commands: Commands,
	cli: Res<Cli>,
	network_channels: Res<NetworkChannels>,
) -> Result<(), Box<dyn Error>> {
	match *cli {
		Cli::Server { port } => {
			let server_channels_config = network_channels.get_server_configs();
			let client_channels_config = network_channels.get_client_configs();

			let server = RenetServer::new(ConnectionConfig {
				server_channels_config,
				client_channels_config,
				..Default::default()
			});

			let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
			let public_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
			let socket = UdpSocket::bind(public_addr)?;
			let server_config = ServerConfig {
				current_time,
				max_clients: 10,
				protocol_id: PROTOCOL_ID,
				authentication: ServerAuthentication::Unsecure,
				public_addresses: vec![public_addr],
			};
			let transport = NetcodeServerTransport::new(server_config, socket)?;

			commands.insert_resource(server);
			commands.insert_resource(transport);

			commands.spawn(TextBundle::from_section(
				"Server",
				TextStyle {
					font_size: 30.0,
					color: Color::WHITE,
					..default()
				},
			));
			// commands.spawn(PlayerBundle::new(SERVER_ID, Vec2::ZERO, Color::GREEN));
		}
		Cli::Client { port, ip } => {
			let server_channels_config = network_channels.get_server_configs();
			let client_channels_config = network_channels.get_client_configs();

			let client = RenetClient::new(ConnectionConfig {
				server_channels_config,
				client_channels_config,
				..Default::default()
			});

			let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
			let client_id = current_time.as_millis() as u64;
			let server_addr = SocketAddr::new(ip, port);
			let socket = UdpSocket::bind((ip, 0))?;
			let authentication = ClientAuthentication::Unsecure {
				client_id,
				protocol_id: PROTOCOL_ID,
				server_addr,
				user_data: None,
			};
			let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

			commands.insert_resource(client);
			commands.insert_resource(ClientIdResource(client_id));
			commands.insert_resource(transport);

			commands.spawn(TextBundle::from_section(
				format!("Client: {client_id:?}"),
				TextStyle {
					font_size: 30.0,
					color: Color::WHITE,
					..default()
				},
			));
		}
	}

	Ok(())
}

fn server_event_system(mut commands: Commands, mut server_event: EventReader<ServerEvent>) {
	for event in server_event.read() {
		match event {
			ServerEvent::ClientConnected { client_id } => {
				info!("player: {client_id} Connected");
				// Generate pseudo random color from client id.
				commands.spawn((
					Player(client_id.clone()),
					Replication,
					Transform::from_xyz(0., 0., 0.)
				)
				);
			}
			ServerEvent::ClientDisconnected { client_id, reason } => {
				info!("client {client_id} disconnected: {reason}");
			}
		}
	}
}


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
	client_id: Res<ClientIdResource>,
) {
	for (player_position, player) in &player_position_query {
		if player.0.raw() == client_id.0 {
			let mut transform = camera_query.get_single_mut().unwrap();

			transform.translation = transform
				.translation
				.lerp(player_position.extend(1.).as_vec3(), time.delta_seconds() * 10.);
		}
	}	
}
