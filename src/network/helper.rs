use std::marker::PhantomData;

use bevy::ecs::schedule::SystemSet;
use bevy::ecs::system::{Resource, Res};

use bevy::utils::{HashMap, HashSet};
use bevy_simplenet::ChannelPack;

use bevy_simplenet::Client;

#[cfg(feature = "server")]
use bevy_simplenet::Server;
use serde::{Serialize, Deserialize};

use super::tick::RepliconTick;

pub const SERVER_ID: ClientId = 0;

#[derive(Resource, Copy, Debug)]
#[derive(Clone)]
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
pub struct ServerMsg(pub ChannelId, pub RepliconTick, pub Vec<u8>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientMsg(pub ChannelId, pub Vec<u8>);


#[derive(Debug, Clone)]
pub struct NetworkChannel;
impl ChannelPack for NetworkChannel
{
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

