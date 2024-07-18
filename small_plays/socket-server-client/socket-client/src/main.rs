use std::io::prelude::*;
use std::net::TcpStream;
use bincode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct DummyProtocol {
    request_type: RequestType,
    asset: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum RequestType {
    GET,
    POST,
    PUT,
    DELETE,
}


fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    let dummy_protocol = DummyProtocol {
        request_type: RequestType::GET,
        asset: "dummy".to_string(),
    };

    let data = bincode::serialize(&dummy_protocol).unwrap();
    println!("{:?}", data);
    let temp1 = &data;
    let temp2: &[u8] = temp1;
    //let data_len: u64 = data.len().try_into().unwrap();
    stream.write(&temp2)?;
    Ok(())
}
