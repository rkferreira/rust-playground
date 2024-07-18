use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use serde::{Deserialize, Serialize};
use serde_json;
use bincode;

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

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        dummy_handle_connection(stream);
    }
}

fn dummy_handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut buf = Vec::new();
    buf_reader.read_to_end(&mut buf).unwrap();
    println!("Connection established");
    println!("{:?}", buf_reader);

    println!("{:?}", buf);
    let dummy: DummyProtocol = bincode::deserialize(&buf).unwrap();
    //let dummy: DummyProtocol = serde_json::from_reader(buf_reader).unwrap();
    println!("{:?}", dummy);
}

fn http_handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream.write_all(response.as_bytes()).unwrap();
}
