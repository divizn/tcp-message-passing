use core::panic;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::env::args;
use std::io::stdin;
use std::thread;

fn main() {
    let server_addr = args().nth(1).expect("Please provide server IP");
    println!("{server_addr}");

    let mut buf: [u8; 256] = [0; 256];
    let mut stream = TcpStream::connect(&server_addr).expect("Failed to connect");
    let mut input_stream = stream.try_clone().expect("Failed to clone socket");
    
    thread::spawn(move || { 
        loop {
            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(0) => break,
                
                Ok(_) => {
                    input_stream.write(input.as_bytes()).expect("Failed to read buffer to stream");
                    input.clear();
                },
                
                Err(e) => panic!("Could not read from stdin: {e}"),            
            };
        }});

    loop {
        match stream.read(&mut buf) {
        Ok(n) => {
            let message: String = String::from_utf8(buf[0..n].to_vec()).unwrap();
            
            println!("{message}");
        },

        Err(e) => panic!("{e}"),
    }};

}
