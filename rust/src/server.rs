use std::io::{Read, Write};
use std::env::args;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::Ipv4Addr;


use sysinfo::System;

const GIGABYTE: f32 = 1000000000.0;
const PORT: &str = "6969";

struct SystemUsage<'a> {
    cpu_usage: f32,
    memory_usage: f32,
    system: &'a mut System,
}

impl<'a> SystemUsage<'a> {
    fn refresh(&mut self) {
        let sys = &mut self.system;
        // thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL); // making sure cpu usage is up to date
        sys.refresh_all();

        let cpu_usage: f32 = sys.global_cpu_info().cpu_usage() as f32;
        let memory_usage: f32 = (sys.used_memory() as f32 / GIGABYTE) / (sys.total_memory() as f32 / GIGABYTE) * 100.0;
        
        self.cpu_usage = cpu_usage;
        self.memory_usage = memory_usage;
    }

    fn show(&self, ctx: &str) {
        let cpu_usage = self.cpu_usage;
        let memory_usage = self.memory_usage;
        println!("{ctx}:");
        println!("Memory usage: {memory_usage:.2}%"); println!("CPU usage: {cpu_usage:.2}%");
    }
}

fn main() {
    let mut sys = System::new_all();
    let mut sys = SystemUsage {
        cpu_usage: sys.global_cpu_info().cpu_usage() as f32,
        memory_usage: (sys.used_memory() as f32 / GIGABYTE) / (sys.total_memory() as f32 / GIGABYTE) * 100.0,
        system: &mut sys,
    };
    sys.show("Start of program");

    let ip = get_ip(&mut sys);

    println!("Creating socket at: {ip}");

    let connections: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    let server = match TcpListener::bind(&ip) {
        Ok(server) => {
            println!("Listening at {ip}");
            sys.refresh();
            sys.show("After binding to socket");
            server
        },
        Err(_) => {
            println!("Could not create a socket at {ip}, trying a different port");
            let mut new_ip: String;
            let port: usize;
            new_ip = ip.split(":").next().expect("IP is invalid").to_string() + ":"; 
            port = ip.split(":").nth(1).expect("Port is invalid").parse::<usize>().expect("Port is not a number") + 1;
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
                sys.refresh();
                sys.show(format!("After accepting new connection ({} total connection(s)", connections.lock().unwrap().len() + 1).as_str());
                let addr = stream.peer_addr().expect("Unable to read peer address");
                let connections_clone = Arc::clone(&connections);
                thread::spawn(move || {
                    let mut sys = System::new_all();
                    let mut sys = SystemUsage {
                        cpu_usage: sys.global_cpu_info().cpu_usage() as f32,
                        memory_usage: (sys.used_memory() as f32 / GIGABYTE) / (sys.total_memory() as f32 / GIGABYTE) * 100.0,
                        system: &mut sys,
                    };
                    sys.show("After spawning thread");
                    handle_connection(stream, addr, &connections_clone, &mut sys);
                    sys.refresh();
                    sys.show(format!("After closing connection ({} total connection(s)", connections_clone.lock().unwrap().len()).as_str());
                });
            },
            Err(e) => panic!("{e}"),
        }
    }

}

fn handle_connection(mut stream: TcpStream, addr: SocketAddr, connections: &Arc<Mutex<Vec<TcpStream>>>, sys: &mut SystemUsage) {
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
                if buffer[0] == 10 && n == 1 { continue }
                let mut inp = &buffer[0..n];
                // println!("Received {inp:?} from {addr}");
                println!("Received {n} bytes from {addr}");
                sys.refresh();
                sys.show("After receiving data");

                if inp[n-1] == 10 { inp = inp.strip_suffix(&[10]).unwrap()}
                let out_str = addr.to_string() + ": " + &String::from_utf8(inp.to_vec()).expect("Could not convert to string");
                let out = out_str.as_bytes();
                // println!("Sending {out:?} from {addr}");
                println!("Sending {} bytes from {addr}", out.len());
                let streams = connections.lock().expect("Unable to lock streams");
                for mut connection in streams.iter() {
                    if connection.peer_addr().unwrap() != addr {
                        connection.write_all(out).unwrap();
                    }
                }
                sys.refresh();
                sys.show(format!("After sending data (to {} client(s)", streams.len() - 1).as_str());
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

fn get_ip(sys: &mut SystemUsage) -> String {
    sys.refresh();
    sys.show("Before getting IP from args");
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
    sys.refresh();
    sys.show("After getting IP from args (and parsing)");
    ip
}
