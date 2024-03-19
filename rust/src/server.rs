use std::io::{Read, Write};
use std::env::args;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::Ipv4Addr;


const PORT: &str = "8000";


fn main() {
    let ip = get_ip();

    println!("Creating socket at: {ip}");

    let connections: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    let server = match TcpListener::bind(&ip) {
        Ok(server) => {
            println!("Listening at {ip}");
            server
        },
        Err(_) => {
            println!("Could not create a socket at {ip}, trying a different port");
            let mut new_ip: String;
            new_ip = ip.split(':').next().expect("IP is invalid").to_string() + ":"; 
            let port = ip.split(':').nth(1).expect("Port is invalid").parse::<usize>().expect("Port is not a number") + 1;
            new_ip += port.to_string().as_str();

            println!("Creating socket at: {new_ip}");
            match TcpListener::bind(&new_ip) {
                Ok(server) => {
                    println!("Listening at {new_ip}");
                    server
                },
                Err(e) => panic!("Error creating socket, maybe try another port\n{e}")
            }
        }
    };

    for connection in server.incoming() {
        match connection {
            Ok(stream) => {
                let addr = stream.peer_addr().expect("Unable to read peer address");
                let connections_clone = Arc::clone(&connections);
                thread::spawn(move || {
                    handle_connection(stream, addr, &connections_clone);
                });
            },
            Err(e) => panic!("{e}"),
        }
    }

}

fn handle_connection(mut stream: TcpStream, addr: SocketAddr, connections: &Arc<Mutex<Vec<TcpStream>>>) {
    println!("Accepted connection from: {addr}");

    {
        connections.lock().unwrap().push(stream.try_clone().expect("Could not clone client stream"));
    }

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection has been closed by {addr}");
                break
            },
            Ok(n) => {
                println!("Received {n} bytes from {addr}");
                let Ok(out_str) = generate_output(n, &buffer, addr) else { continue };
                let out = out_str.as_bytes();


                println!("Sending {} bytes from {addr}", out.len());
                let streams = connections.lock().expect("Unable to lock streams");
                for mut connection in streams.iter() {
                    if connection.peer_addr().unwrap() != addr {
                        connection.write_all(out).unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from socket: {e}");
                break;
            }
        }
    }

    {
        connections.lock().unwrap().retain(|streams| {
            addr != streams.peer_addr().expect("Could not retrieve peer address")
        });
    }
}

fn generate_output(n: usize, buffer: &[u8; 1024], addr: SocketAddr) -> Result<String, ()> {
    if buffer[0] == 10 && n == 1 { return Err(()) }
    let mut inp = &buffer[0..n];
    
    if inp[n-1] == 10 { inp = inp.strip_suffix(&[10]).unwrap()}
    let out = addr.to_string() + ": " + &String::from_utf8(inp.to_vec()).expect("Could not convert to string");
    Ok(out)
}

fn get_ip() -> String {
    let args: Vec<String> = args().collect();
    let mut ip: String;

    if let Some(arg_ip) = args.get(1) {
        if let Ok(ip_addr) = arg_ip.parse::<Ipv4Addr>() {
            ip = ip_addr.to_string() + ":";
        } else {
            println!("Invalid IPv4 address provided, using 127.0.0.1");
            ip = String::from("127.0.0.1:");
        }
    } else {
        println!("No IPv4 address provided, using 127.0.0.1");
        ip = String::from("127.0.0.1:");
    }
    
    if let Some(arg_port) = args.get(2) {
        if let Ok(port_number) = arg_port.parse::<u16>() {
            ip += &port_number.to_string();
        } else {
            ip += PORT;
            println!("Invalid port number provided, using 8000");
        }
    } else {
        ip += PORT;
        println!("No port number provided, using 8000");
    }
    ip
}
