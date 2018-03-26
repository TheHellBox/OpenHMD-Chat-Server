pub mod client_params;

use std::collections::HashMap;
use bytevec::{ByteEncodable, ByteDecodable};
use cobalt;
use player;
use support;
use std::thread;
use std::sync::mpsc;

use cobalt::{
    BinaryRateLimiter, Config, NoopPacketModifier, MessageKind, UdpSocket,
    Server, ServerEvent
};

pub struct Network {
    pub server: Server<UdpSocket,BinaryRateLimiter,NoopPacketModifier>,
    //                   Data   Client  Type  MessageKind
    pub tx: mpsc::Sender<(Vec<u8>, u32, u8, MessageKind)>,
    pub rx: mpsc::Receiver<(Vec<u8>, u32, u8, MessageKind)>,
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
        use std::time::Duration;
        use std::sync::mpsc::channel;
        let (tx, rx) = channel::<(Vec<u8>, u32, u8, MessageKind)>();
        let mut config = Config::default();
        config.connection_closing_threshold = Duration::from_millis(5000);
        config.connection_drop_threshold = Duration::from_millis(5000);
        config.connection_init_threshold = Duration::from_millis(5000);
        config.send_rate = 1024;
        let server = Server::<UdpSocket, BinaryRateLimiter, NoopPacketModifier>::new(config);

        Network{
            server: server,
            tx: tx,
            rx: rx
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
    pub fn send_assets_to(&mut self, id: cobalt::ConnectionID, params: &client_params::ClParams){
        use std::io::prelude::*;
        use std::fs::File;
        use std::io::BufReader;
        use std::fs;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let tx = self.tx.clone();
        let params = params.gamefiles.clone();
        // Spawning a new thread, otherwise it will block network thread
        thread::spawn(move ||{
            //Get textures pathes
            let textures = fs::read_dir("./assets/textures/").unwrap();
            //Get models pathes
            let models = fs::read_dir("./assets/models/").unwrap();

            let mut pathes = vec![];

            let mut hasher = DefaultHasher::new();

            for x in textures{
                let path = x.unwrap().path();
                if path.is_file(){
                    //Checking if format is ok
                    if path.extension().unwrap() == "png" || path.extension().unwrap() == "jpeg"{
                        pathes.push(path);
                    }
                }
            }
            for x in models{
                let path = x.unwrap().path();
                if path.is_file(){
                    //Checking if format is ok
                    if path.extension().unwrap() == "obj" || path.extension().unwrap() == "mtl"{
                        pathes.push(path);
                    }
                }
            }
            //Converting ConnectionID to u32
            let cobalt::ConnectionID(id) = id;

            tx.send((vec![200, 237], id,5,cobalt::MessageKind::Ordered));
            thread::sleep_ms(20);
            //Sending all files
            for filename in pathes{
                //Opening file
                let mut file = File::open(&filename).unwrap();
                let mut buf = vec![];
                file.read_to_end(&mut buf);
                let hash = buf.hash(&mut hasher);
                let name = filename.display().to_string();
                let mut file = BufReader::new(buf.as_slice());
                if !(params.get(&name).is_some() && params.get(&name).unwrap() == &format!("{}", hasher.finish())) {
                    //That byte means that it start of file stream
                    let mut startmsg = vec![233, 144, 122, 198, 134, 253, 251];
                    startmsg.append(&mut filename.display().to_string().as_bytes().to_vec());
                    //Sending to main thread
                    tx.send((startmsg, id,5,cobalt::MessageKind::Ordered));
                    thread::sleep_ms(100);
                    //Send file
                    loop{
                        let mut buf = &mut [0;1024];
                        let mut data = file.read(buf);
                        if data.is_ok(){
                            let data = data.unwrap();
                            if data != 0{
                                tx.send((buf[0..data].to_vec(), id,5,cobalt::MessageKind::Ordered));
                            }
                            else{
                                break
                            }
                        }else{
                            break
                        }
                    }
                }
            }
            thread::sleep_ms(100);
            tx.send((vec![100, 137, 211, 233, 212, 222], id,5,cobalt::MessageKind::Ordered));
        });
    }

    pub fn accept(&mut self, players: &mut HashMap<u32, player::Player>, map: &support::map_loader::Map){
        while let Ok(event) = self.server.accept_receive() {
            match event{
                ServerEvent::Message(id, message) => {
                    match message[0]{
                        // Different actions to different msg types, first byte is a type
                        0 => {
                            //println!("{:?}", &message[1..message.len()])
                        },
                        1 => {
                            //self.send_expect(id, message[1..message.len()].to_vec(), *&message[0], cobalt::MessageKind::Instant);
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
                        4 => {
                            let params = client_params::ClParams::from_network(message[1..message.len()].to_vec());
                            let cobalt::ConnectionID(rid) = id;
                            for (obj_id, x) in map.objects(){
                                let mut data = x.to_network();
                                self.tx.send((data, rid,4, cobalt::MessageKind::Reliable));
                            }
                            for (col_id, x) in map.colliders(){
                                let mut data = x.to_network();
                                self.tx.send((data, rid,6, cobalt::MessageKind::Reliable));
                            }
                            self.send_assets_to(id, &params);
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
                _ => {}
            }
        };
        //FIXME: Poor code... Maybe...
        let mut msgs = vec![];
        {
            for x in self.rx.try_iter(){
                msgs.push(x);
            }
        }
        for x in msgs{
            let (data, client, type_d, msgk) = x;
            self.send_to(cobalt::ConnectionID(client), data, type_d, msgk);
        }
        // Ping
        self.server.send(true).is_ok();
    }
}
