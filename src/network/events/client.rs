use crate::network::channels_config::ChannelManager;
use crate::network::helper::*;
use bevy::prelude::*;
use bincode::{DefaultOptions, Options};
use serde::{de::DeserializeOwned, Serialize};

pub trait ClientEventAppExt {
	fn add_client_event<T: Event + Serialize + DeserializeOwned>(&mut self) -> &mut Self;
}

impl ClientEventAppExt for App {
	fn add_client_event<T: Event + Serialize + DeserializeOwned>(&mut self) -> &mut Self {
		// Create Event Channel Resource
		// init resouce Events<FromClient<T>>
		// Add event <T>
		// Add reciving system that runs  before update
		// Add sending system that runs after update
		let channel_id = self
			.world
			.resource_mut::<ChannelManager>()
			.create_client_channel();

		self.add_event::<T>()
			.init_resource::<Events<FromClient<T>>>()
			.insert_resource(EventChannel::<T>::new(channel_id));

		#[cfg(feature = "server")]
		self.add_systems(
			PreUpdate,
			receiving_system::<T>
				.in_set(ServerSet::Receive)
				.run_if(resource_exists::<ServerSn>()),
		);

		self.add_systems(
			PostUpdate,
			(
				sending_system::<T>.run_if(client_connected()),
				local_resending_system::<T>.run_if(has_authority()),
			)
				.chain()
				.in_set(ClientSet::Send),
		);

		self
	}
}

#[derive(Clone, Copy, Event, Debug)]
pub struct FromClient<T> {
	pub client_id: u128,
	pub event: T,
}

#[cfg(feature = "server")]
fn receiving_system<T: Event + DeserializeOwned>(
	mut server: ResMut<ServerSn>,
	mut client_events: EventWriter<FromClient<T>>,
	channel: Res<EventChannel<T>>,
) {
	if let Some(message_queue) = server.message_channel_buckets.get_mut(&channel.channel_id) {
		for (client_id, client_message) in message_queue.drain(0..) {
			let event: T = DefaultOptions::new()
				.deserialize(&client_message.1)
				.expect("server should send valid events");

			client_events.send(FromClient { client_id, event })
		}
	}
}

fn sending_system<T: Event + Serialize>(
	mut events: EventReader<T>,
	client: ResMut<ClientSn>,
	channel: Res<EventChannel<T>>,
) {
	for event in events.read() {
		let message = DefaultOptions::new()
			.serialize(&event)
			.expect("client event should be serializable");

		client
			.simplenet
			.send(ClientMsg(channel.channel_id, message))
			.unwrap();
	}
}

/// Transforms `T` events into [`FromClient<T>`] events to "emulate"
/// message sending for offline mode or when server is also a player
fn local_resending_system<T: Event>(
	mut events: ResMut<Events<T>>,
	mut client_events: EventWriter<FromClient<T>>,
) {
	for event in events.drain() {
		client_events.send(FromClient {
			client_id: SERVER_ID,
			event,
		})
	}
}
