use std::collections::{HashMap, HashSet};
use std::io;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::Duration;
use serde::Serialize;

mod duration_float {
    use std::fmt::Formatter;
    use std::time::Duration;

    use serde::{Deserializer, Serializer};
    use serde::de::{Error, Visitor};

    pub fn serialize<S>(val: &Duration, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_f64(val.as_secs_f64())
    }

    struct Visit;

    impl Visitor<'_> for Visit {
        type Value = Duration;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            write!(formatter, "a positive duration as a decimal number")
        }

        fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
            (v.is_finite() && v > 0.0)
                .then_some(Duration::from_secs_f64(v))
                .ok_or(E::custom("duration must be positive, non-zero, and finite"))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_f64(Visit)
    }
}

/// Configuration for a server.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub(crate) path: PathBuf,
    /// How long the server will wait for a client to respond to a ping.
    #[serde(with = "duration_float")]
    pub packet_timeout: Duration,
    /// How often the server will send pings to clients.
    #[serde(with = "duration_float")]
    pub ping_spacing: Duration,
    /// The default world to connect to.
    pub default_world: String,
    /// The IP to connect to.
    pub ip: IpAddr,
    /// A mapping of banned IPs to their ban reasons.
    pub banned_ips: HashMap<IpAddr, String>,
    /// A mapping of banned usernames to their ban reasons.
    pub banned_users: HashMap<String, String>,
    /// A set of usernames that are operators.
    pub operators: HashSet<String>,
    /// The amount of salts to keep for verifying users.
    ///
    /// If this is zero, then users will not be verified.
    pub kept_salts: usize,
    /// The server name to display in the server list.
    pub name: String,
    /// A URL linking to the heartbeat server the server will ping.
    ///
    /// If this is empty, then the heartbeat URL will not be pinged.
    ///
    /// Note that leaving this empty AND setting `kept_salts` to above 0
    /// will create a situation where players will not be able to be
    /// verified! This will cause a runtime error.
    pub heartbeat_url: String,
    /// How often the server will send pings to the heartbeat server.
    #[serde(with = "duration_float")]
    pub heartbeat_spacing: Duration,
    /// How long the server will wait for sending pings to the heartbeat server before trying again.
    #[serde(with = "duration_float")]
    pub heartbeat_timeout: Duration,
    /// The port to host the server on.
    pub port: u16,
    /// The maximum amount of players allowed on the server.
    ///
    /// If this is set to 0, then the amount will be unlimited.
    pub max_players: usize,
    /// Whether the server should be public in the server list.
    pub public: bool,
    /// The server's MOTD.
    pub motd: String,
    /// The maximum message length.
    pub max_message_length: usize
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: PathBuf::default(),
            packet_timeout: Duration::from_secs(10),
            ping_spacing: Duration::from_millis(500),
            default_world: "default".into(),
            banned_ips: HashMap::from([
                (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), "<ban reason>".into()),
                (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 1)), "<ban reason>".into()),
                (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 2)), "<ban reason>".into())
            ]),
            banned_users: HashMap::from([
                ("Alice".into(), "<ban reason>".into()),
                ("Bob".into(), "<ban reason>".into()),
                ("Charlie".into(), "<ban reason>".into())
            ]),
            ip: IpAddr::from([127, 0, 0, 1]),
            kept_salts: 0,
            name: "<Unnamed Server>".to_string(),
            heartbeat_url: String::new(),
            heartbeat_spacing: Duration::from_secs(5),
            heartbeat_timeout: Duration::from_secs(5),
            port: 25565,
            max_players: 64,
            public: false,
            operators: HashSet::new(),
            motd: "Running on Honeybit".into(),
            max_message_length: 256
        }
    }
}

static COMMENT_MAP: [(&str, &str); 17] = [
    ("packet_timeout", "How long the server should wait before disconnecting a player, in seconds."),
    ("ping_spacing", "How often the server sends pings to clients, in seconds."),
    ("default_world", "The world that players first connect to when joining."),
    ("operators", "A list of usernames that have operator permissions."),
    ("kept_salts", "How many \"salts\" to keep in memory.\nSalts are used to verify a user's key.\nIf this is set to 0, then users will not be verified."),
    ("name", "The server's displayed name."),
    ("heartbeat_url", "The URL to ping for heartbeat pings.\n\nIf this is left blank, then no heartbeat pings will be sent.\nIf this is left blank AND kept_salts is above 0,\nthe program will exit with an error,\nas it will be impossible for users to join."),
    ("heartbeat_spacing", "How often heartbeat pings will be sent, in seconds."),
    ("heartbeat_timeout", "How long the server will wait to hear back from the heartbeat server, in seconds."),
    ("ip", "The IP to listen for connections on."),
    ("port", "The port to host the server on."),
    ("max_players", "The maximum amount of players on the server."),
    ("public", "Whether the server will show as public on the heartbeat URLs corresponding server list."),
    ("motd", "The server's MOTD."),
    ("max_message_length", "The maximum length of a sent message. Messages above this threshold will be clipped."),
    ("[banned_ips]", "A mapping of IPs to ban reasons."),
    ("[banned_users]", "A mapping of usernames to ban reasons."),
];

impl Config {
    pub fn save(&self, buf: &mut String) -> io::Result<()> {
        self.serialize(toml::Serializer::pretty(buf))
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
        // Insert documentation to the config file

        let mut concat = Vec::new();

        for line in buf.lines() {
            let mut commented = false;
            for (prefix, comment) in COMMENT_MAP {
                if line.starts_with(prefix) {
                    for comment_line in comment.lines() {
                        concat.push("# ");
                        concat.push(comment_line);
                        concat.push("\n");
                    }
                    commented = !prefix.starts_with('[');
                    break;
                }
            }
            concat.push(line);
            concat.push("\n");
            if commented {
                concat.push("\n");
            }
        }

        *buf = concat.join("");

        Ok(())
    }
}