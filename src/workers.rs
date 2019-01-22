use std::net::SocketAddr;

mod redirect;
mod store;

struct FuncOpts {
   packet_limit: Option<usize>,
   opponent_addr: Option<SocketAddr>,
}

pub use self::redirect::redirect;
pub use self::store::store;
