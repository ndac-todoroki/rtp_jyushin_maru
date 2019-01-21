use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

#[macro_use]
extern crate log;

use config;

use postgres;
use postgres::{Connection, TlsMode};

use chrono::prelude::*;
use rtp_rs::RtpReader;

// Struct mapping to tables

/// `test_cases` table
struct TestCase {
    id: uuid::Uuid,

    name: String,

    inserted_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

/// `rtps` table
struct RTP {
    id: uuid::Uuid,
    serial: i64,
    test_case_id: uuid::Uuid,

    version: i32,
    padding: bool,
    extension: bool,
    csrc_count: i32,
    marker: bool,
    payload_type: i32,
    timestamp: i64,
    ssrc: i32,
    payload: Vec<u8>,
    received_at: i32,

    inserted_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

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

fn main() -> std::io::Result<()> {
    env_logger::init();

    /* Settings */

    // Get Postgres settings.
    let mut psql_settings = config::Config::default();
    match psql_settings.merge(config::File::with_name("config/postgres")) {
        Ok(_) => println!("OK: Postgres settings read."),
        Err(err) => {
            eprintln!("error: {:?}", err);
            std::process::exit(1);
        }
    };

    // Get udp settings.
    let mut udp_settings = config::Config::default();
    match udp_settings.merge(config::File::with_name("config/udp")) {
        Ok(_) => println!("OK: UDP settings read."),
        Err(err) => {
            eprintln!("error: {:?}", err);
            std::process::exit(1);
        }
    };
    let host = udp_settings.get_str("host").unwrap();
    let port = udp_settings.get_int("port").unwrap();
    let max_packets = udp_settings.get_int("max_packets").unwrap();
    let opponent_addr: Option<SocketAddr> = None;

    /* UDP Server */

    let socket = UdpSocket::bind(format!("{}:{}", host, port)).expect("couldn't bind to address");
    debug!("Socket binded correctly to {:?}.", socket);

    let mut buf = [0; 2000];

    /* Postgres */

    // Connect to Postgres
    let psql_params = construct_connection_params(&psql_settings);
    let conn = Connection::connect(psql_params, TlsMode::None).unwrap();
    debug!("connected to Postgres");

    // Create message sender/receiver
    let (tx, rx) = mpsc::channel();
    debug!("tx, rx created");

    /* Do. */

    // Create test case: Exit if failed.
    let current_datetime = Utc::now();
    let test_case = TestCase {
        id: uuid::Uuid::new_v4(),
        name: current_datetime.format("%+").to_string(),
        inserted_at: current_datetime.naive_utc(),
        updated_at: current_datetime.naive_utc(),
    };
    conn.execute(
        "INSERT INTO test_cases (id, name, inserted_at, updated_at) VALUES ($1, $2, $3, $4)",
        &[
            &test_case.id,
            &test_case.name,
            &test_case.inserted_at,
            &test_case.updated_at,
        ],
    )
    .expect("CREATING TEST CASE FAILED");

    println!("The case name is: {}", test_case.name);

    // Accept UDP packets and store them with timestamp.
    let handler = thread::spawn(move || {
        debug!("-- inside thread");

        let mut counter = 0;
        let system_time = SystemTime::now();

        loop {
            debug!("loop #{}", counter);

            match socket.recv_from(&mut buf) {
                Ok((recv_size, src_addr)) => {
                    debug!("OK");

                    // Skip if not from ESP32
                    if let Some(opponent_addr) = opponent_addr {
                        if opponent_addr != src_addr {
                            continue;
                        };
                    }

                    let timestamp: i32 = system_time.elapsed().unwrap().subsec_micros() as i32;
                    let current_time = Utc::now().naive_utc();

                    // FIXME: unsafe
                    let rtp = RtpReader::new(&buf[..recv_size]).unwrap();

                    let data = RTP {
                        id: uuid::Uuid::new_v4(),
                        serial: counter,
                        test_case_id: test_case.id,
                        version: rtp.version() as i32,
                        padding: rtp.padding(),
                        extension: rtp.extension(),
                        csrc_count: rtp.csrc_count() as i32,
                        marker: rtp.mark(),
                        payload_type: rtp.payload_type() as i32,
                        timestamp: rtp.timestamp() as i64,
                        ssrc: rtp.ssrc() as i32,
                        payload: rtp.payload().to_vec(),
                        received_at: timestamp,
                        inserted_at: current_time,
                        updated_at: current_time,
                    };
                    tx.send(data).unwrap();

                    // update counter if only OK
                    counter = counter + 1;
                }
                Err(e) => {
                    debug!("recv function failed: {:?}", e);
                    continue;
                }
            };

            // end if max packets count
            if counter >= max_packets {
                break;
            };
        }
    });

    for rtp in rx {
        // println!("Got: {}", received);
        debug!("Got: {}", rtp.id);
        conn.execute(
            "INSERT INTO rtps (id, serial, test_case_id, version, padding, extension, csrc_count, marker, payload_type, timestamp, ssrc, payload, received_at, inserted_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
            &[
                &rtp.id,
                &rtp.serial,
                &rtp.test_case_id,
                &rtp.version,
                &rtp.padding,
                &rtp.extension,
                &rtp.csrc_count,
                &rtp.marker,
                &rtp.payload_type,
                &rtp.timestamp,
                &rtp.ssrc,
                &rtp.payload,
                &rtp.received_at,
                &rtp.inserted_at,
                &rtp.updated_at,
            ],
        ).expect("INSERTING RTP FAILED");
    }

    // wait until all ends
    handler.join().unwrap();

    Ok(())
}
