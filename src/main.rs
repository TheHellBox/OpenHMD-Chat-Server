pub extern crate cobalt;
#[macro_use]
pub extern crate bytevec;

mod network;
mod player;

use std::collections::HashMap;

fn main() {
    let ip = "127.0.0.1";
    let port = 4587;
    println!("Starting server on {}:{}...", ip, port);
    let mut server = network::Network::new();
    let mut playerlist: HashMap<u32, player::Player>  = HashMap::with_capacity(128);
    server.listen(ip, port);
    loop{
        for (id, player) in &playerlist{
            server.send_expect(cobalt::ConnectionID(*id), player.to_network(), 2, cobalt::MessageKind::Instant);
        }
        server.accept(&mut playerlist);
    }
}
