use bevy::prelude::*;
use bincode::{DefaultOptions, Options};
// use ordered_multimap::ListOrderedMultimap;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use crate::network::{
	channels_config::ChannelManager,
	helper::{client_connected, ClientId, ClientSet, ClientSn, EventChannel, ServerMsg, SERVER_ID, GetData},
	// tick::{LastRepliconTick, MinRepliconTick, RepliconTick, self},
};

#[cfg(feature = "server")]
use crate::network::helper::{ServerSet, ServerSn};

/// An extension trait for [`App`] for creating server events.
pub trait ServerEventAppExt {
	/// Registers event `T` that will be emitted on client after sending [`ToClients<T>`] on server.
	fn add_server_event<T: Event + Serialize + DeserializeOwned + Clone + Debug>(&mut self) -> &mut Self;
}

impl ServerEventAppExt for App {
	fn add_server_event<T: Event + Serialize + DeserializeOwned + Clone + Debug>(&mut self) -> &mut Self {
		// self.add_server_event_with::<T, _, _>(, )

		let channel_id = self
			.world
			.resource_mut::<ChannelManager>()
			.create_server_channel();

		dbg!("server channel", std::any::type_name::<T>(), channel_id);

		self.add_event::<T>()
			.init_resource::<Events<ToClient<T>>>()
			// .init_resource::<ServerEventQueue<T>>()
			.insert_resource(EventChannel::<T>::new(channel_id))
			.add_systems(
				PreUpdate,
				receiving_system::<T>
					// .after(ClientPlugin::replication_receiving_system)
					.in_set(ClientSet::Receive)
					.run_if(client_connected()),
			);

		#[cfg(feature = "server")]
		self.add_systems(
			PostUpdate,
			(
				(sending_system::<T>
					.run_if(resource_exists::<ServerSn>()),)
					.chain()
					// .before(ServerPlugin::replication_sending_system)
					.in_set(ServerSet::Send),
				// reset_system::<T>.run_if(resource_removed::<ClientSn>()),
			),
		);

		self
	}
}

/// Applies all queued events if their tick is less or equal to [`LastRepliconTick`].
// fn queue_system<T: Event>(
//     last_tick: Res<LastRepliconTick>,
//     mut server_events: EventWriter<T>,
//     mut client: ResMut<ClientSn>,
// ) {
//     while event_queue
//         .front()
//         .filter(|(&tick, _)| tick <= **last_tick)
//         .is_some()
//     {
//         let (_, event) = event_queue.pop_front().unwrap();
//         server_events.send(event);
//     }
// }

// fn deserialize_event_messages<T: DeserializeOwned>(message: Vec<u8>) -> T {
//     DefaultOptions::new()
//         .deserialize(&message)
//         .expect("server should send valid events")
// }

fn receiving_system<T: Event + DeserializeOwned + Debug>(
	mut server_events: EventWriter<ToClient<T>>,
	mut client: ResMut<ClientSn>,
	// last_tick: Res<LastRepliconTick>,
	channel: Res<EventChannel<T>>,
) {
	if let Some(server_messages) = client.message_channel_buckets.get_mut(&channel.channel_id) {
		for server_msg in server_messages.drain(..) {
			dbg!(server_msg.get_event::<T>());
			server_events.send(ToClient { mode: None, event: server_msg.get_event::<T>() });
		}
	}
}

#[cfg(feature = "server")]
fn sending_system<T: Event + Serialize + Clone + Debug>(
	mut server: ResMut<ServerSn>,
	mut server_events: EventReader<ToClient<T>>,
	// tick: Res<RepliconTick>,
	channel: Res<EventChannel<T>>,
) {
	for ToClient { event, mode } in server_events.read() {
		send_server_event::<T>(&mut server, channel.channel_id, mode.unwrap(), event.clone());
	}
}

/// Updates [`MinRepliconTick`] to force server to send replication message even if there were no world changes.
///
/// Needed because events on a client won't be emitted until the client acknowledges the event tick.
/// See also [`ServerEventQueue`].
// fn min_tick_update_system<T: Event>(
// 	mut server_events: EventReader<ToClients<T>>,
// 	mut min_tick: ResMut<MinRepliconTick>,
// 	tick: Res<RepliconTick>,
// ) {
// 	if server_events.read().count() > 0 {
// 		**min_tick = *tick;
// 	}
// }

/// Transforms [`ToClients<T>`] events into `T` events to "emulate"
/// message sending for offline mode or when server is also a player
// fn local_resending_system<T: Event>(
//     mut server_events: ResMut<Events<ToClients<T>>>,
//     mut local_events: EventWriter<T>,
// ) {
//     for ToClients { event, mode } in server_events.drain() {
//         match mode {
//             SendMode::Broadcast => {
//                 local_events.send(event);
//             }
//             SendMode::BroadcastExcept(client_id) => {
//                 if client_id != SERVER_ID {
//                     local_events.send(event);
//                 }
//             }
//             SendMode::Direct(client_id) => {
//                 if client_id == SERVER_ID {
//                     local_events.send(event);
//                 }
//             }
//         }
//     }
// }

// fn reset_system<T: Event>(mut event_queue: ResMut<ServerEventQueue<T>>) {
//     event_queue.clear();
// }

/// Sends serialized `message` to clients.
///
/// Helper for custom sending systems.
// /// See also [`ServerEventAppExt::add_server_event_with`]
use crate::network::helper::ChannelId;

#[cfg(feature = "server")]
pub fn send_server_event<T: Serialize + Debug>(
	server: &mut ServerSn,
	channel_id: ChannelId,
	mode: SendMode,
    // tick: Option<RepliconTick>,
	message: T,
) {
	dbg!("Sending server event");
	dbg!(&message);

    let message = DefaultOptions::new()
        .serialize(&message)
        .expect("server event should be serializable");	

    match mode {
		SendMode::Broadcast => {
			for client_id in server.client_connections.iter() {
				server
					.simplenet
					.send(
						*client_id,
						ServerMsg {
                            channel_id,
                            event: message.clone(),
                        }
					)
					.unwrap();
			}
		}
		SendMode::BroadcastExcept(except_client_id) => {
			for client_id in server.client_connections.iter() {
				if *client_id == except_client_id {
					continue;
				}
				server
					.simplenet
					.send(
						*client_id,
						ServerMsg {
                            channel_id,
                            // tick,
                            event: message.clone(),
                        }
                    )
					.unwrap();
			}
		}
		SendMode::Direct(client_id) => {
			if client_id != SERVER_ID {
				server
					.simplenet
					.send(
                        client_id,
                        ServerMsg {
                            channel_id,
                            // tick,
                            event: message.clone(),
                        }
                    )
					.unwrap();
			}
		}
	}
}

/// An event that will be send to client(s).
#[derive(Clone, Copy, Debug, Event)]
pub struct ToClient<T> {
	pub mode: Option<SendMode>,
	pub event: T,
}

/// Type of server message sending.
#[derive(Clone, Copy, Debug)]
#[allow(unused)]
pub enum SendMode {
	Broadcast,
	BroadcastExcept(ClientId),
	Direct(ClientId),
}

// Stores all received events from server that arrived earlier then replication message with their tick.
//
// Stores data sorted by ticks and maintains order of arrival.
// Needed to ensure that when an event is triggered, all the data that it affects or references already exists.
// #[derive(Deref, DerefMut, Resource)]
// pub struct ServerEventQueue<T>(ListOrderedMultimap<RepliconTick, T>);

// impl<T> Default for ServerEventQueue<T> {
// 	fn default() -> Self {
// 		Self(Default::default())
// 	}
// }
