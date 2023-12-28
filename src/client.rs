use core::panic;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::env::args;
use std::thread;
use std::io::stdin;

fn main() {
    let server_addr = args().nth(1).expect("Please provide server IP");
    println!("{server_addr}");

    let mut buf: [u8; 256] = [0; 256];

    println!("{server_addr}");
    let mut stream = TcpStream::connect(&server_addr).expect(server_addr.as_str());
    // let mut strin: &TcpStream = &stream;

        loop {
            match stream.read(&mut buf) {
                Ok(n) => {
                    let message: String = buf[0..n]
                    .iter()
                    .map(|&b| {
                        b.to_ascii_lowercase() as char
                    }).collect();
                    
                    println!("{message:?}")
                },
                Err(e) => panic!("{e}"),
            }
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(0) => break,

            Ok(n) => { 
                println!("{n} bytes read");
                println!("{input}");
                stream.write_all(input.as_bytes()).expect("Failed to read buffer to stream");
            },

            Err(e) => panic!("Could not read from stdin: {e}"),            
        }
    }

}
