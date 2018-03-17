#[macro_use]
pub extern crate bytevec;

pub extern crate cobalt;
pub extern crate rand;
pub extern crate json;

mod network;
mod player;
mod support;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let ip = "0.0.0.0";
    let port = 4587;
    println!("Starting server on {}:{}...", ip, port);
    let mut server = network::Network::new();
    let mut playerlist: HashMap<u32, player::Player>  = HashMap::with_capacity(128);
    server.listen(ip, port);

    println!("Loading map...");
    let mut file = File::open("./assets/maps/simple_scene.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let mut map = support::map_loader::Map::new();
    map.load(&content);
    println!("Loading finished!");
    loop{
        for (id, player) in &playerlist{
            server.send_expect(cobalt::ConnectionID(*id), player.to_network(), 2, cobalt::MessageKind::Instant);
        }
        server.accept(&mut playerlist, &map);
    }
}
