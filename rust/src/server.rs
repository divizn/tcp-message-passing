use std::io::{Read, Write};
use std::env::args;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::Ipv4Addr;

const PORT: &str = "6969";

fn main() {

    let ip = get_ip();

    println!("Creating socket at: {ip}");

    let connections: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    let server = match TcpListener::bind(&ip) {
        Ok(server) => server,
        Err(eip) => panic!("Incorrect socket address configuration! Please change IP: {eip}")
    };

    println!("Listening at {ip}");

    for connection in server.incoming() {
        match connection {
            Ok(stream) => {
                let addr = stream.peer_addr().expect("Unable to read peer address");
                let connections_clone = Arc::clone(&connections);
                thread::spawn(move || {
                    handle_connection(stream, addr, &connections_clone);
                }); //maybe rc with all streams so i can send message from one to all
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
            Ok(0) => {println!("Connection has been closed by {addr}"); break},
            Ok(n) => {
                if buffer[0] == 10 && n == 1 { continue }
                let mut inp = &buffer[0..n];
                println!("Received {inp:?} from {addr}"); 
                if inp[n-1] == 10 { inp = inp.strip_suffix(&[10]).unwrap()}

                let out_str = addr.to_string() + ": " + &String::from_utf8(inp.to_vec()).expect("Could not convert to string");
                let out = out_str.as_bytes();
                println!("Sending {out:?} from {addr}");
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
            addr != streams.peer_addr().expect("Could not retrieve peer address") // temp fix?
        });
    }
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
            println!("Invalid port number provided, using 6969");
        }
    } else {
        ip += PORT;
        println!("No port number provided, using 6969");
    }
    ip
}
