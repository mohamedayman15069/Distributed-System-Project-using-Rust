use std::thread;
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use nix::unistd::{fork, ForkResult};

fn main() {
    //will change to external IP later
    let socket_external = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            
            
            let socket_internal = UdpSocket::bind("127.0.0.1:4245").expect("couldn't bind to address");

            let listener_addresses = ["127.0.0.1:4242", "127.0.0.1:4243", "127.0.0.1:4244"];
            let mut i=0;
            loop{
                let mut buf = [0;3];
                let (number_of_bytes, client_address) = socket_internal.recv_from(&mut buf)
                                                        .expect("Didn't receive data");
                // thread::spawn(move || {
                    let socket_clone = socket_external.try_clone().expect("couldn't clone the socket");
                    let client_port_num = client_address.port();
                    let mut buf_agent = [0;5];
                    for j in 0..3 {
                        buf_agent[j]=buf[j];
                    }
                    buf_agent[4] = client_port_num as u8;       //LSB
                    buf_agent[3] = (client_port_num>>8) as u8;  //MSB
                    socket_clone.send_to(&buf_agent, listener_addresses[i%3].to_string()).expect("couldn't send data");
                    println!("i: {}",i);
                    println!("Arr: {:?}",buf_agent);
                    println!("Port: {:?}",((buf_agent[3] as u16) << 8) | (buf_agent[4] as u16));
                    i += 1;
                    i %= 3;
                // });
            }
        }
        Ok(ForkResult::Child) => {
            
            let socket_internal = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            let mut buf_agent_reply = [0;5];
            loop {
                let (number_of_bytes, _) = socket_external.recv_from(&mut buf_agent_reply)
                                                .expect("Didn't receive data");
                // thread::spawn(move || {
                    let client_port: u16 = ((buf_agent_reply[3] as u16) << 8) | (buf_agent_reply[4] as u16);
                    let socket_address_client_reply = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), client_port);
                    let mut buf = [0;5];
                    for j in 0..3 {
                        buf[j]=buf_agent_reply[j];
                    }
                    socket_internal.send_to(&buf, socket_address_client_reply).expect("couldn't send data");
                // });
            }
        }
        Err(_) => {
            // Error
            println!("Error");
        }
    }
}

// We need to fork the client into two porgrams, one for the recieving requests from clinets and
// forwarding to servers (see above). The other is for recieving replies from servers and 
// forwarding to correct clients (see below).

// Each will have an ifinite loop that will be waiting on a recieve on its internal/external
// sockets respectively.

/*
let mut buf_agent_reply = [0;5];
let (number_of_bytes, _) = socket_external.recv_from(&mut buf_agent_reply)
                                .expect("Didn't receive data");
//We need to find a way to use the port number in buf
//alongside localhost IP in order to use instead of placeholder below
let client_port: u16 = ((buf_agent_reply[3] as u16) << 8) | (buf_agent_reply[4] as u16);
let socket_address_client_reply = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), client_port);
let mut buf = [0;5];
for j in 0..3 {
    buf[j]=buf_agent_reply[j];
}
socket_internal.send_to(&buf_agent_reply, socket_address_client_reply).expect("couldn't send data");
*/