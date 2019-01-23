use std::net::SocketAddr;

mod redirect;
mod store;

pub struct FuncOpts {
   pub packet_limit: Option<usize>,
   pub opponent_addr: Option<SocketAddr>,
}

pub use self::redirect::redirect;
pub use self::store::store;
