use std::marker::PhantomData;

use bevy::ecs::schedule::SystemSet;
use bevy::ecs::system::{Res, Resource};

use bevy::utils::{HashMap, HashSet};
use bevy_simplenet::ChannelPack;

use bevy_simplenet::Client;

#[cfg(feature = "server")]
use bevy_simplenet::Server;
use bincode::{DefaultOptions, Options as _};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const SERVER_ID: ClientId = 0;

#[derive(Resource, Clone, Copy, Debug)]
pub struct EventChannel<T> {
	pub channel_id: ChannelId,
	pub marker: PhantomData<T>,
}

impl<T> EventChannel<T> {
	pub fn new(channel_id: ChannelId) -> Self {
		Self {
			channel_id,
			marker: PhantomData,
		}
	}
}

pub type ClientId = u128;
pub type ChannelId = u8;

/// Set with replication and event systems related to server.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ServerSet {
	PreRecieve,
	/// Systems that receive data.
	///
	/// Runs in `PreUpdate`.
	Receive,
	/// Systems that send data.
	///
	/// Runs in `PostUpdate` on server tick, see [`TickPolicy`].
	Send,
}

/// Set with replication and event systems related to client.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ClientSet {
	PreReceive,
	/// Systems that receive data.
	///
	/// Runs in `PreUpdate`.
	Receive,
	/// Systems that send data.
	///
	/// Runs in `PostUpdate`.
	Send,
}

// define a channel
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectMsg(pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerMsg {
    pub channel_id: ChannelId, 
    // pub tick: Option<RepliconTick>, 
    pub event: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientMsg {
    pub channel_id: ChannelId, 
    pub event: Vec<u8>
}

pub trait GetData {
    fn get_bytes(&self) -> Vec<u8>;
    fn from_bytes(message: Vec<u8>, channel_id: ChannelId) -> Self;
    fn get_event<T: DeserializeOwned>(&self) -> T;
}

impl GetData for ServerMsg {
    fn get_bytes(&self) -> Vec<u8> {
        DefaultOptions::new().serialize(&self.event.clone()).expect("serializable")
    }

    fn from_bytes(message: Vec<u8>, channel_id: ChannelId) -> Self {
        let event = DefaultOptions::new()
            .deserialize(&message)
            .expect("server should send valid events");

        Self { channel_id, event }
    }

    fn get_event<T: DeserializeOwned>(&self) -> T {
        DefaultOptions::new()
            .deserialize(&self.event)
            .expect("server should send valid events")
    }
}

impl GetData for ClientMsg {
    fn get_bytes(&self) -> Vec<u8> {
        self.event.clone()
    }

    fn from_bytes(message: Vec<u8>, channel_id: ChannelId) -> Self {
        Self { channel_id, event: message }
    }

    fn get_event<T: DeserializeOwned>(&self) -> T {
        DefaultOptions::new()
            .deserialize(&self.event)
            .expect("server should send valid events")
    }
}

#[derive(Debug, Clone)]
pub struct NetworkChannel;
impl ChannelPack for NetworkChannel {
	type ConnectMsg = ConnectMsg;
	type ServerMsg = ServerMsg;
	type ServerResponse = ();
	type ClientMsg = ClientMsg;
	type ClientRequest = ();
}

// type TestClientEvent = ClientEventFrom<NetworkChannel>;
// type TestServerEvent = ServerEventFrom<NetworkChannel>;

// fn server_factory() -> ServerFactory<NetworkChannel>
// {
//     // It is recommended to make server/client factories with baked-in protocol versions (e.g.
//     //   with env!("CARGO_PKG_VERSION")).
//     ServerFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION"))
// }

// fn client_factory() -> ClientFactory<NetworkChannel>
// {
//     // You must use the same protocol version string as the server factory.
//     ClientFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION"))
// }

// Sn = Simplenet
#[derive(Resource)]
pub struct ClientSn {
	pub simplenet: Client<NetworkChannel>,
	pub message_channel_buckets: HashMap<ChannelId, Vec<ServerMsg>>,
}

#[cfg(feature = "server")]
#[derive(Resource)]
pub struct ServerSn {
	pub simplenet: Server<NetworkChannel>,
	pub message_channel_buckets: HashMap<ChannelId, Vec<(ClientId, ClientMsg)>>,
	pub client_connections: HashSet<u128>,
}

pub fn client_connected() -> impl FnMut(Option<Res<ClientSn>>) -> bool {
	|client| match client {
		Some(client) => client.simplenet.is_connected(),
		None => false,
	}
}

#[allow(unused)]
pub fn client_disconnected() -> impl FnMut(Option<Res<ClientSn>>) -> bool {
	|client| match client {
		Some(client) => client.simplenet.is_connected(),
		None => true,
	}
}

/// Condition that returns `true` for server or in singleplayer and `false` for client.
pub fn has_authority() -> impl FnMut(Option<Res<ClientSn>>) -> bool + Clone {
	move |client| client.is_none()
}

