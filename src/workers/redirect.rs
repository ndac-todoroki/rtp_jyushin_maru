use super::FuncOpts;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;

use rtp_rs::RtpReader;

pub fn redirect(socket: UdpSocket, opts: FuncOpts) {
   use std::io::{self, Write};

   let mut buf = [0; 2000];
   let mut stdout = io::stdout();

   // Create message sender/receiver
   let (tx, rx) = mpsc::channel();
   debug!("tx, rx created");

   /* Do. */

   // Accept UDP packets and take payloads out.
   let handler = thread::spawn(move || {
      let mut counter = 0usize;

      loop {
         match socket.recv_from(&mut buf) {
            Ok((recv_size, src_addr)) => {
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

               // FIXME: unsafe
               let rtp = RtpReader::new(&buf[..recv_size]).unwrap();

               tx.send(rtp.payload().to_vec()).unwrap();

               // update counter if only OK
               counter = counter + 1;
            }
            Err(_) => {
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

   for payload in rx {
      debug!("Got: {:?}", payload);
      stdout.write_all(&payload);
   }

   // wait until all ends
   handler.join().unwrap();
}
