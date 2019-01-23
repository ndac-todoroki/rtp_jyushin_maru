use std::net::{SocketAddr, UdpSocket};

use clap::*;
use config;
use log::debug;
use postgres::{self, Connection, TlsMode};

// Local modules
mod model;
mod workers;

/// Constructs a connection parameter from the config file.
/// This `panic!`s if failed to read some values.
fn construct_connection_params(settings: &config::Config) -> postgres::params::ConnectParams {
   use postgres::params::{ConnectParams, Host};

   let mut params = ConnectParams::builder();

   if let Ok(user) = settings.get_str("user") {
      let pass = match settings.get_str("password") {
         Ok(password) => Some(password),
         Err(_) => None,
      };
      params.user(&user, pass.as_ref().map(String::as_str));
   };

   if let Ok(database) = settings.get_str("database") {
      params.database(&database);
   };

   params.build(Host::Tcp(settings.get_str("host").unwrap()))
}

fn main() {
   use self::workers::{redirect, store, FuncOpts};
   use directories::ProjectDirs;

   let project_dirs = ProjectDirs::from(
      "org",            /*qualifier*/
      crate_authors!(), /*organization*/
      crate_name!(),    /*application*/
   )
   .unwrap();

   env_logger::init();

   /* Command-line arguments */

   let mut app = clap_app!((crate_name!()) =>
      (version: crate_version!())
      (author: crate_authors!())
      (about: "AES67 packet receiver. It can store packets to Postgres, or redirect payloads to STDOUT, or else.")
      (@arg CONFIG: -c --config +takes_value +global "A config file defining the UDP port settings.")
      (@arg limit: -l --limit +takes_value +global "If set, it only reads the given number of packets and exits.")
      (@subcommand store =>
         (about: "Stores packets to Postgres.")
         (@arg postgres_config: -p --("postgres-config") +required +takes_value "Config file for PostgreSQL")
      )
      (@subcommand redirect =>
         (about: "Redirects packet payload to stdout. Good for pipeline use.")
      )
   );

   let matches = app.clone().get_matches();

   // get config
   let default_config_path = project_dirs
      .config_dir()
      .join("udp.yml")
      .to_string_lossy()
      .to_string();
   let udp_config_file_path = matches.value_of("CONFIG").unwrap_or(&default_config_path);

   let packet_limit = match matches.value_of("limit") {
      Some(limit) => match limit.parse::<usize>() {
         Ok(limit) => Some(limit),
         Err(_) => None,
      },
      None => None,
   };

   // Get udp settings.
   let mut udp_settings = config::Config::default();
   match udp_settings.merge(config::File::with_name(udp_config_file_path)) {
      Ok(_) => debug!("OK: UDP settings read."),
      Err(err) => {
         eprintln!("error: {:?}", err);
         std::process::exit(1);
      }
   };
   let host = udp_settings.get_str("host").unwrap();
   let port = udp_settings.get_int("port").unwrap();
   let opponent_addr: Option<SocketAddr> = None;

   /* UDP Server */

   let socket = UdpSocket::bind(format!("{}:{}", host, port)).expect("couldn't bind to address");
   debug!("Socket binded correctly to {:?}.", socket);

   match matches.subcommand() {
      ("store", Some(sub_match)) => {
         let psql_config_file_path = sub_match.value_of("postgres_config").unwrap();
         // Get Postgres settings.
         let mut psql_settings = config::Config::default();
         match psql_settings.merge(config::File::with_name(psql_config_file_path)) {
            Ok(_) => debug!("OK: Postgres settings read."),
            Err(err) => {
               eprintln!("error: {:?}", err);
               std::process::exit(1);
            }
         };
         // Connect to Postgres
         let psql_params = construct_connection_params(&psql_settings);
         let conn = Connection::connect(psql_params, TlsMode::None).unwrap();
         debug!("connected to Postgres");

         store(
            socket,
            conn,
            FuncOpts {
               packet_limit,
               opponent_addr,
            },
         );
      }
      ("redirect", Some(_)) => redirect(
         socket,
         FuncOpts {
            packet_limit,
            opponent_addr,
         },
      ),
      _ => {
         app.print_help();
      }
   }
}
