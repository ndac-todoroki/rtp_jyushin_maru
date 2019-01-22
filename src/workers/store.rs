use super::FuncOpts;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use postgres;

use chrono::prelude::*;
use rtp_rs::RtpReader;

pub fn store(socket: UdpSocket, conn: postgres::Connection, opts: FuncOpts) {
   use crate::model::{TestCase, RTP};

   let mut buf = [0; 2000];

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
   conn
      .execute(
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

      let mut counter = 0usize;
      let system_time = SystemTime::now();

      loop {
         debug!("loop #{}", counter);

         match socket.recv_from(&mut buf) {
            Ok((recv_size, src_addr)) => {
               debug!("OK");

               // Skip if not from ESP32
               if let FuncOpts {
                  opponent_addr: Some(addr),
                  ..
               } = opts
               {
                  if addr != src_addr {
                     continue;
                  };
               }

               let timestamp: i32 = system_time.elapsed().unwrap().subsec_micros() as i32;
               let current_time = Utc::now().naive_utc();

               // FIXME: unsafe
               let rtp = RtpReader::new(&buf[..recv_size]).unwrap();

               let data = RTP {
                  id: uuid::Uuid::new_v4(),
                  serial: counter as i64,
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
         if let FuncOpts {
            packet_limit: Some(packet_limit),
            ..
         } = opts
         {
            if counter >= packet_limit {
               break;
            };
         }
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
}
