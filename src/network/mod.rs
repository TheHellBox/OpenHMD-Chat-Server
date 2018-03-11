use std::collections::HashMap;
use bytevec::{ByteEncodable, ByteDecodable};
use cobalt;
use player;

use cobalt::{
    BinaryRateLimiter, Config, NoopPacketModifier, MessageKind, UdpSocket,
    Server, ServerEvent
};

pub struct Network {
    pub server: Server<UdpSocket,BinaryRateLimiter,NoopPacketModifier>
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct NetAudio {
    data: Vec<u8>,
    id: u32
}
bytevec_impls! {
    impl NetAudio {
        data: Vec<u8>,
        id: u32
    }
}
impl NetAudio {
    pub fn to_network(&self) -> Vec<u8>{
        self.encode::<u8>().unwrap()
    }
    pub fn from_network(message: Vec<u8>) -> NetAudio{
        NetAudio::decode::<u8>(&message).unwrap()
    }
}

impl Network {
    pub fn new() -> Network{
        let mut server = Server::<UdpSocket, BinaryRateLimiter, NoopPacketModifier>::new(Config::default());

        Network{
            server: server
        }
    }
    pub fn listen(&mut self, ip: &'static str, port: u32){
        self.server.listen(&format!("{}:{}", ip, port)).expect("Failed to bind to socket.");
    }
    pub fn send(&mut self, msg: Vec<u8>, type_d: u8, type_m: MessageKind){
        let mut msg = msg;
        msg.insert(0, type_d);
        for (_, conn) in self.server.connections() {
            conn.send(type_m, msg.clone());
        }
    }
    pub fn send_to(&mut self, id: cobalt::ConnectionID, msg: Vec<u8>, type_d: u8, type_m: MessageKind){
        let mut msg = msg;
        msg.insert(0, type_d);
        for (_, conn) in self.server.connections() {
            if conn.id() == id{
                conn.send(type_m, msg.clone());
            }
        }
    }
    pub fn send_expect(&mut self, id: cobalt::ConnectionID, msg: Vec<u8>, type_d: u8, type_m: MessageKind){
        let mut msg = msg;
        msg.insert(0, type_d);
        for (_, conn) in self.server.connections() {
            if conn.id() != id{
                conn.send(type_m, msg.clone());
            }
        }
    }
    pub fn accept(&mut self, players: &mut HashMap<u32, player::Player>){
        while let Ok(event) = self.server.accept_receive() {
            match event{
                ServerEvent::Message(id, message) => {
                    match message[0]{
                        0 => {
                            println!("{:?}", &message[1..message.len()])
                        },
                        1 => {
                            self.send_expect(id, message[1..message.len()].to_vec(), *&message[0], cobalt::MessageKind::Instant);
                        },
                        2 => {
                            let cobalt::ConnectionID(id) = id;
                            let mut player = player::Player::from_network(message[1..message.len()].to_vec());
                            player.id = id;
                            players.insert(id, player);
                        },
                        3 => {
                            let cobalt::ConnectionID(nid) = id;
                            let mut data = NetAudio::from_network(message[1..message.len()].to_vec());
                            data.id = nid;
                            let data = data.to_network();
                            self.send_expect(id, data, *&message[0], cobalt::MessageKind::Instant);
                        },
                        _ => {}
                    }
                },
                ServerEvent::Connection(rid) => {
                    let cobalt::ConnectionID(id) = rid;
                    println!("Player {} connected!", id);
                },
                ServerEvent::ConnectionLost(id, status) => {
                    let cobalt::ConnectionID(id) = id;
                    println!("Player {} disconnected! Reason: ConnectionLost", id);
                    players.remove(&id);
                },
                ServerEvent::ConnectionClosed(id, status) => {
                    let cobalt::ConnectionID(id) = id;
                    println!("Player {} disconnected! Reason: ConnectionClosed", id);
                    players.remove(&id);
                },
                _ => println!("{:?}", event)
            }
        };
        self.server.send(true).is_ok();
    }
}
