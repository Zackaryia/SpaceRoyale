use bevy::prelude::*;

pub struct ServerChannelConfig {
    
}

/// ID of the server replication channel.
///
/// See also [`NetworkChannels`].
// pub const REPLICATION_CHANNEL_ID: u8 = 0;

/// A resource to configure and setup channels for [`ConnectionConfig`](bevy_renet::renet::ConnectionConfig)
#[derive(Clone, Resource)]
pub struct ChannelManager {
	// Stores delivery guarantee and maximum usage bytes (if set) for each server channel.
	// server: Vec<Option<usize>>,
	server: u8,

	// Same as [`Self::server`], but for client.
	// client: Vec<(SendType, Option<usize>)>,
	client: u8,
	// Stores default max memory usage bytes for all channels.
	//
	// This value will be used instead of `None`.
	// default_max_bytes: usize,
}

/// Stores only replication channel by default.
impl Default for ChannelManager {
	fn default() -> Self {
		Self {
			server: 0,
			client: 0,
			// default_max_bytes: 5 * 1024 * 1024, // Value from `DefaultChannel::config()`.
		}
	}
}

impl ChannelManager {
	// /// Returns server channel configs that can be used to create [`ConnectionConfig`](bevy_renet::renet::ConnectionConfig).
	// pub fn get_server_configs(&self) -> Vec<ChannelConfig> {
	//     self.get_configs(&self.server)
	// }

	// /// Same as [`Self::get_server_configs`], but for client.
	// pub fn get_client_configs(&self) -> Vec<ChannelConfig> {
	//     self.get_configs(&self.client)
	// }

	// /// Sets maximum usage bytes for specific client channel.
	// ///
	// /// [`REPLICATION_CHANNEL_ID`] or [`EventChannel<T>`](crate::network_event::EventChannel) can be passed as `id`.
	// /// Without calling this function, the default value will be used.
	// /// See also [`Self::set_default_max_bytes`].
	// pub fn set_server_max_bytes(&mut self, id: impl Into<u8>, max_bytes: usize) {
	//     let id = id.into() as usize;
	//     let (_, bytes) = self
	//         .server
	//         .get_mut(id)
	//         .unwrap_or_else(|| panic!("there is no server channel with id {id}"));

	//     *bytes = Some(max_bytes);
	// }

	// /// Same as [`Self::set_server_max_bytes`], but for client.
	// pub fn set_client_max_bytes(&mut self, id: impl Into<u8>, max_bytes: usize) {
	//     let id = id.into();
	//     let (_, bytes) = self
	//         .client
	//         .get_mut(id as usize)
	//         .unwrap_or_else(|| panic!("there is no client channel with id {id}"));

	//     *bytes = Some(max_bytes);
	// }

	// /// Sets maximum usage bytes that will be used by default for all channels if not set.
	// pub fn set_default_max_bytes(&mut self, max_bytes: usize) {
	//     self.default_max_bytes = max_bytes;
	// }

	pub(super) fn create_client_channel(&mut self) -> u8 {
		if self.client == u8::MAX {
			panic!("number of client channels shouldn't exceed u8::MAX");
		}

		self.client += 1;
		self.client
	}

	pub(super) fn create_server_channel(&mut self) -> u8 {
		if self.server == u8::MAX {
			panic!("number of server channels shouldn't exceed u8::MAX");
		}

		self.server += 1;
		self.server
	}

	// fn get_configs(&self, channels: &[(SendType, Option<usize>)]) -> Vec<ChannelConfig> {
	//     let mut channel_configs = Vec::with_capacity(channels.len());
	//     for (index, (send_type, max_bytes)) in channels.iter().enumerate() {
	//         channel_configs.push(ChannelConfig {
	//             channel_id: index as u8,
	//             max_memory_usage_bytes: max_bytes.unwrap_or(self.default_max_bytes),
	//             send_type: send_type.clone(),
	//         });
	//     }
	//     channel_configs
	// }
}
