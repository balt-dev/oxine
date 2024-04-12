//! Handles the actual server.

use std::{
    collections::{
        HashMap,
        HashSet, VecDeque
    }, net::IpAddr, time::Duration
};
use crate::world::World;
use rand::{rngs::StdRng, Rng};

/// A trait to help generate valid salts for the server.
pub trait SaltExt {
    /// Generate a salt.
    fn salt(&mut self) -> String;
}

impl SaltExt for StdRng {
    #[inline]
    fn salt(&mut self) -> String {
        let num: u128 = self.gen();
        base62::encode(num)
    }
}

/// An instance of a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// A mapping of names to worlds in the server.
    pub worlds: HashMap<String, World>,
    /// The configuration for the server.
    pub config: Config,
    /// The last few salts generated by the server. The length is dictated by the server configuration.
    pub last_salts: VecDeque<String>,
    /// A mapping of player names to which world the player is connected to and which ID the player is in that world.
    pub players_connected: HashMap<String, (String, i8)>
}

impl Server {
    /// Disconnect a player from the server by username.
    /// 
    /// This does not close the player's networking loops!
    pub fn disconnect(&mut self, username: impl AsRef<str>) {
        let world = self.players_connected.remove(username.as_ref());
        if let Some((world, id)) = world.and_then(
            |(world, id)| self.worlds.get_mut(&world).map(
                |w| (w, id)
            )
        ) {
            world.remove_player(id);
        }
    }
}


/// Configuration for a server.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// How long the server will wait for a client to respond to a packet.
    pub packet_timeout: Duration,
    /// How often the server will send pings to clients.
    pub ping_spacing: Duration,
    /// The default world to connect to.
    pub default_world: String,
    /// A list of banned IPs.
    pub banned_ips: HashSet<IpAddr>,
    /// The amount of salts to keep for verifying users.
    /// 
    /// If this is zero, then users will not be verified.
    pub kept_salts: usize,
    /// The server name to display in the server list.
    pub name: String,
    /// A URL linking to the heartbeat server the server will ping.
    pub heartbeat_url: String,
    /// The amount of times to retry connecting to the heartbeat server.
    pub heartbeat_retries: usize,
    /// How often the server will send pings to the heartbeat server.
    pub heartbeat_spacing: Duration,
    /// The port to host the server on.
    pub port: u16,
}