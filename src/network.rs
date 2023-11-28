use bevy_xpbd_2d::prelude::*;

use bevy_simplenet::{
    ChannelPack, ClientEventFrom, ServerEventFrom,
    ServerFactory, ClientFactory, ServerReport, ClientReport,
    AcceptorConfig, Authenticator, ServerConfig, AuthRequest,
    ClientConfig, MessageStatus, RequestStatus, EnvType, Server, Client
};
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

use std::cmp::Ordering;
use crate::*;


pub struct NetworkPlugin;

#[derive(Event)]
pub struct ClientMsgEvent {
    pub msg: ClientMsg,
    pub id: u128
}

#[derive(Event)]
pub struct ServerMsgEvent {
    pub msg: ServerMsg,
    pub id: u128
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, Self::startup)
            .add_event::<ClientMsgEvent>()
            .add_event::<ServerMsgEvent>();

        #[cfg(feature = "server")]
        app.add_systems(FixedUpdate, (Self::send_server_msgs, Self::recieve_client_msgs));
        #[cfg(feature = "client")]
        app.add_systems(FixedUpdate, (Self::send_client_msgs, Self::recieve_server_msgs));

    }
}
impl NetworkPlugin {
    fn startup(mut commands: Commands) {
        #[cfg(feature = "server")]
        {
            let server = ServerFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION")).new_server(
                enfync::builtin::native::TokioHandle::default(),
                "127.0.0.1:38727",
                AcceptorConfig::Default,
                Authenticator::None,
                ServerConfig::default(),
            );

            dbg!(server.url());
            dbg!(server.url().as_str());

            commands.insert_resource(server);
        }
        #[cfg(feature = "client")]
        {
            let client_id: u128 = rand::random();
            let client = ClientFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION")).new_client(
                enfync::builtin::Handle::default(),  //automatically selects native/WASM runtime
                url::Url::parse("ws://127.0.0.1:38727/ws").unwrap(),
                AuthRequest::None{ client_id },
                ClientConfig::default(),
                ConnectMsg(String::from("hello"))
            );

            commands.insert_resource(client);
        }
    }

    fn send_server_msgs(mut server_events: EventReader<ServerMsgEvent>, server_network: Res<Server<NetworkChannel>>) {
        for event in server_events.read() {
            server_network.send(event.id, event.msg.clone()).unwrap();
        }
    }

    fn send_client_msgs(mut client_events: EventReader<ClientMsgEvent>, client_network: Res<Client<NetworkChannel>>) {
        for event in client_events.read() {
            client_network.send(event.msg.clone()).unwrap();
        }
    }

    fn recieve_server_msgs(mut server_events: EventWriter<ServerMsgEvent>, client_network: Res<Client<NetworkChannel>>) {
        loop {
            let Some(msg) = client_network.next() else { break };
            
            match msg {
                bevy_simplenet::ClientEvent::Report(data) => { dbg!(data); },
                bevy_simplenet::ClientEvent::Msg(msg) => server_events.send(ServerMsgEvent { msg, id: client_network.id() }),
                bevy_simplenet::ClientEvent::Response(_, _) => todo!(),
                bevy_simplenet::ClientEvent::Ack(_) => todo!(),
                bevy_simplenet::ClientEvent::Reject(_) => todo!(),
                bevy_simplenet::ClientEvent::SendFailed(_) => todo!(),
                bevy_simplenet::ClientEvent::ResponseLost(_) => todo!(),
            };
        };
    }

    fn recieve_client_msgs(mut client_events: EventWriter<ClientMsgEvent>, server_network: Res<Server<NetworkChannel>>) {
        loop {
            let Some(full_msg) = server_network.next() else { break };
            
            match full_msg.1 {
                bevy_simplenet::ServerEvent::Report(data) => { dbg!(data); },
                bevy_simplenet::ServerEvent::Msg(msg) => client_events.send(ClientMsgEvent { msg, id: full_msg.0 }),
                bevy_simplenet::ServerEvent::Request(_, _) => todo!(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PlayerState {
    id: u32,
    // powerups: Vec<(PowerupEnum, f32)>,
    transform: Transform,
	l_vel: LinearVelocity,
	a_vel: AngularVelocity,
	ext_f: ExternalForce,
    // health: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GameState {
    tick: RepliconTick,
    players: Vec<PlayerState>,
}


struct BulletState {
    id: u32,
    transform: Transform,
    l_vel: LinearVelocity,
    author: u32,
}

struct GrenadeState {
    id: u32,
    transform: Transform,
    l_vel: LinearVelocity,
    ext_f: ExternalForce,
    author: u32,
}

enum PowerupEnum {
    Health
}

struct PowerupState {
    id: u32,
    transform: Transform,
}

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

type TestClientEvent = ClientEventFrom<NetworkChannel>;
type TestServerEvent = ServerEventFrom<NetworkChannel>;

fn server_factory() -> ServerFactory<NetworkChannel>
{
    // It is recommended to make server/client factories with baked-in protocol versions (e.g.
    //   with env!("CARGO_PKG_VERSION")).
    ServerFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION"))
}

fn client_factory() -> ClientFactory<NetworkChannel>
{
    // You must use the same protocol version string as the server factory.
    ClientFactory::<NetworkChannel>::new(env!("CARGO_PKG_VERSION"))
}

use bevy::prelude::*;


/// Marks entity for replication.
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Replication;

/// A tick that increments each time we need the server to compute and send an update.
///
/// Used as resource only on server.
/// Mapped to the Bevy's `Tick` in [`AckedTicks`](crate::server::AckedTicks).
/// See also [`TickPolicy`](crate::server::TickPolicy).
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Resource, Serialize)]
pub struct RepliconTick(pub(crate) u32);

impl RepliconTick {
    /// Gets the value of this tick.
    #[inline]
    pub fn get(self) -> u32 {
        self.0
    }

    /// Increments current tick by the specified `value` and takes wrapping into account.
    #[inline]
    pub fn increment_by(&mut self, value: u32) {
        self.0 = self.0.wrapping_add(value);
    }

    /// Same as [`Self::increment_by`], but increments only by 1.
    #[inline]
    pub fn increment(&mut self) {
        self.increment_by(1)
    }
}

impl PartialOrd for RepliconTick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let difference = self.0.wrapping_sub(other.0);
        if difference == 0 {
            Some(Ordering::Equal)
        } else if difference > u32::MAX / 2 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}


/// Last received tick from server.
///
/// Used only on clients, sent to the server in last replicon ack message.
#[derive(Debug, Default, Deref, Resource)]
pub struct LastReplicatedTick(pub(super) RepliconTick);


#[derive(Event)]
struct ServerStateUpdateEvent;

#[derive(Event)]
struct UserInputEvent;


