pub mod helper;
// mod events;
pub mod channels_config;
pub mod events;

use bevy::prelude::*;

use bevy::utils::{HashMap, HashSet};
use bevy_simplenet::{AuthRequest, Authenticator};

#[cfg(feature = "client")]
use bevy_simplenet::{ClientConfig, ClientFactory};

#[cfg(feature = "server")]
use bevy_simplenet::{AcceptorConfig, ServerConfig, ServerFactory, ServerReport};
use serde::{Deserialize, Serialize};

use crate::network::helper::NetworkChannel;

use self::channels_config::ChannelManager;
use self::events::server::ServerEventAppExt;
use self::helper::ClientId;
#[cfg(feature = "client")]
use self::helper::{ClientSet, ClientSn, ConnectMsg};

#[cfg(feature = "server")]
use self::helper::{ServerSet, ServerSn};

// use self::tick::{LastRepliconTick, MinRepliconTick, RepliconTick};

#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct EventClientConnected(pub ClientId);
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub struct EventClientDisconnected(pub ClientId);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PreStartup, Self::startup)
			.init_resource::<ChannelManager>()
            .add_server_event::<InternalConnectionEvent>();

		#[cfg(feature = "client")]
		app
			// .init_resource::<LastRepliconTick>()
			.configure_sets(
				PreUpdate,
				ClientSet::Receive, //.after(NetcodeClientPlugin::update_system),
			)
			.configure_sets(
				PostUpdate,
				ClientSet::Send, //.before(NetcodeClientPlugin::send_packets),
			)
			.add_systems(
				PreUpdate,
				Self::client_reciving_messages_bucketer
					.in_set(ClientSet::PreReceive)
					.run_if(resource_exists::<ClientSn>()),
			);

		#[cfg(feature = "server")]
		app
			// .init_resource::<RepliconTick>()
			// .init_resource::<MinRepliconTick>()
			.add_systems(
				PreUpdate,
				Self::server_reciving_messages_bucketer
					.in_set(ServerSet::PreRecieve)
					.run_if(resource_exists::<ServerSn>()),
			)
			.add_event::<EventClientConnected>()
			.add_event::<EventClientDisconnected>();
	}
}

#[derive(Debug, Deserialize, Event, Serialize, Clone)]
pub enum InternalConnectionEvent {
    Connected(ClientId),
    Disconnected(ClientId),
}

impl NetworkPlugin {
	fn startup(mut commands: Commands) {
		#[cfg(feature = "server")]
		{
			let server = ServerFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION"))
				.new_server(
					enfync::builtin::native::TokioHandle::default(),
					"127.0.0.1:38727",
					AcceptorConfig::Default,
					Authenticator::None,
					ServerConfig::default(),
				);

			dbg!(server.url());
			dbg!(server.url().as_str());

			let server = ServerSn {
				simplenet: server,
				message_channel_buckets: HashMap::new(),
				client_connections: HashSet::new(),
			};

			commands.insert_resource(server);
		}
		#[cfg(feature = "client")]
		{
			let client_id: u128 = rand::random();
			let client = ClientFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION"))
				.new_client(
					enfync::builtin::Handle::default(), //automatically selects native/WASM runtime
					url::Url::parse("ws://127.0.0.1:38727/ws").unwrap(),
					AuthRequest::None { client_id },
					ClientConfig::default(),
					ConnectMsg(String::from("hello")),
				);

			let client = ClientSn {
				simplenet: client,
				message_channel_buckets: HashMap::new(),
			};

			commands.insert_resource(client);
		}
	}

	#[cfg(feature = "server")]
	pub fn server_reciving_messages_bucketer(
		mut server: ResMut<ServerSn>, 
		mut client_connected_event: EventWriter<EventClientConnected>,
		mut client_disconnected_event: EventWriter<EventClientDisconnected>,
	) {
        use crate::network::{events::server::send_server_event, channels_config::CONNECTION_EVENT_CHANNEL_ID};

		while let Some((client_id, message)) = server.simplenet.next() {
			match message {
				bevy_simplenet::ServerEvent::Report(report) => {
					dbg!(&report);
					match report {
						ServerReport::Connected(_env, _connection_msg) => {
							assert!(server.client_connections.insert(client_id));
							client_connected_event.send(EventClientConnected(client_id));
                            send_server_event::<InternalConnectionEvent>(
                                &mut server, 
                                CONNECTION_EVENT_CHANNEL_ID, 
                                events::server::SendMode::Broadcast, 
                                InternalConnectionEvent::Connected(client_id)
                            )
						}
						ServerReport::Disconnected => {
							assert!(server.client_connections.remove(&client_id));
							client_disconnected_event.send(EventClientDisconnected(client_id));
                            send_server_event::<InternalConnectionEvent>(
                                &mut server, 
                                CONNECTION_EVENT_CHANNEL_ID, 
                                events::server::SendMode::Broadcast, 
                                InternalConnectionEvent::Disconnected(client_id)
                            )
						}
					}
				}
				bevy_simplenet::ServerEvent::Msg(message) => {
					if server.message_channel_buckets.get(&message.channel_id).is_none() {
						assert!(server
							.message_channel_buckets
							.insert(message.channel_id, Vec::new())
							.is_none());
					}
					server
						.message_channel_buckets
						.get_mut(&message.channel_id)
						.unwrap()
						.push((client_id, message));
				}
				bevy_simplenet::ServerEvent::Request(_, _) => todo!(),
			}
		}
	}

	#[cfg(feature = "client")]
	pub fn client_reciving_messages_bucketer(mut client: ResMut<ClientSn>) {
		while let Some(message) = client.simplenet.next() {
			match message {
				bevy_simplenet::ClientEvent::Report(data) => {
					dbg!(data);
				}
				bevy_simplenet::ClientEvent::Msg(message) => {
					if client.message_channel_buckets.get(&message.channel_id).is_none() {
						assert!(client
							.message_channel_buckets
							.insert(message.channel_id, Vec::new())
							.is_none());
					}

					client
						.message_channel_buckets
						.get_mut(&message.channel_id)
						.unwrap()
						.push(message);
				}
				bevy_simplenet::ClientEvent::Response(_, _) => todo!(),
				bevy_simplenet::ClientEvent::Ack(_) => todo!(),
				bevy_simplenet::ClientEvent::Reject(_) => todo!(),
				bevy_simplenet::ClientEvent::SendFailed(_) => todo!(),
				bevy_simplenet::ClientEvent::ResponseLost(_) => todo!(),
			}
		}
	}
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// struct PlayerState {
//     id: u32,
//     // powerups: Vec<(PowerupEnum, f32)>,
//     transform: Transform,
// 	l_vel: LinearVelocity,
// 	a_vel: AngularVelocity,
// 	ext_f: ExternalForce,
//     // health: f32,
// }

// #[derive(Serialize, Deserialize, Clone, Debug)]
// struct GameState {
//     tick: RepliconTick,
//     players: Vec<PlayerState>,
// }

// struct BulletState {
//     id: u32,
//     transform: Transform,
//     l_vel: LinearVelocity,
//     author: u32,
// }

// struct GrenadeState {
//     id: u32,
//     transform: Transform,
//     l_vel: LinearVelocity,
//     ext_f: ExternalForce,
//     author: u32,
// }

// enum PowerupEnum {
//     Health
// }

// struct PowerupState {
//     id: u32,
//     transform: Transform,
// }
