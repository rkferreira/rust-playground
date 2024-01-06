#![feature(asm)]

// Code was taken from some random tutorial on the internet
// don't remember where exactly, but it was a good tutorial
// if you claim its yours, I'll give you credit

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use core::arch::asm;

fn handle_read(mut stream: &TcpStream) {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            println!("{}", req_str);
        }
        Err(e) => println!("Unable to read stream: {}", e),
    }
}

fn handle_write(mut stream: TcpStream) {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html;
charset=UTF-8\r\n\r\nHello world\r\n";
    match stream.write(response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }

  unsafe {
        asm!("CLI");
        asm!("HLT");
    }

}
fn handle_client(stream: TcpStream) {
    handle_read(&stream);
    handle_write(stream);
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    println!("Listening for connections on port {}", 80);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
