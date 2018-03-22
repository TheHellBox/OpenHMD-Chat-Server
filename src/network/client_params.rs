
use bytevec::ByteEncodable;
use bytevec::ByteDecodable;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Default, Clone)]
pub struct ClParams{
    pub version: (u32, u32),
    pub gamefiles: HashMap<String, String>,
}

bytevec_impls! {
    impl ClParams {
        version: (u32, u32),
        gamefiles: HashMap<String, String>
    }
}
impl ClParams {
    pub fn new() -> ClParams{
        ClParams{
            version: (0, 0),
            gamefiles: HashMap::new()
        }
    }
    pub fn to_network(&self) -> Vec<u8>{
        self.encode::<u16>().unwrap()
    }
    pub fn from_network(message: Vec<u8>) -> ClParams{
        ClParams::decode::<u16>(&message).unwrap()
    }
}
