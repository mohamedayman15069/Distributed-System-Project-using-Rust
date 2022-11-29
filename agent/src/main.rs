use std::{thread, time};
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use nix::unistd::{fork, ForkResult};
use queues::*;
use std::sync::{Arc, Mutex};

fn main() {
    //will change to external IP later
    let socket_external = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    let active = [1;3];
    let listener_addresses = ["127.0.0.1:4242", "127.0.0.1:4243", "127.0.0.1:4244"];
    let mut address_to_idx = HashMap::new();
    for i in 0..3 {
        address_to_idx.insert(listener_addresses[i as usize], i);
    }
    let (mut reader, mut writer) = os_pipe::pipe().unwrap();
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            drop(writer);
            

            let mut outbound_requests: Queue<[u8;5]> = queue![];
            
            let user_original = Arc::new(Mutex::new(outbound_requests));
            
            
            
            let mut i=0;
            let user1 = user_original.clone();
            thread::spawn(move || {
                loop{
                    let mut locked_user = user1.lock().unwrap();
                    if locked_user.size()>0
                    {
                        let temp: [u8;5] = locked_user.peek().unwrap();
                        locked_user.remove();
                        drop(locked_user);
                        socket_external.send_to(&temp, listener_addresses[i]).expect("couldn't send data");
                        i += 1;
                        i %= 3;
                    }
                    else
                    {
                        drop(locked_user);
                        thread::sleep(time::Duration::from_millis(10));
                    }
                }
            });
            
            thread::spawn(move || {
                loop{
                    let mut buf = [0;2];   //////
                    reader.read(&mut buf).unwrap();
                    active[buf[0] as idx] = buf[1];
                }
            });

            let socket_internal = UdpSocket::bind("127.0.0.1:4245").expect("couldn't bind to address");

            
            loop{
                let mut buf = [0;3];
                let (number_of_bytes, client_address) = socket_internal.recv_from(&mut buf)
                                                        .expect("Didn't receive data");
                let user = user_original.clone();
                thread::spawn(move || {
                    // let socket_clone = socket_external.try_clone().expect("couldn't clone the socket");
                    let client_port_num = client_address.port();
                    let mut buf_agent = [0;5];
                    for j in 0..3 {
                        buf_agent[j]=buf[j];
                    }
                    buf_agent[4] = client_port_num as u8;       //LSB
                    buf_agent[3] = (client_port_num>>8) as u8;  //MSB
                    let mut locked_user = user.lock().unwrap();
                    locked_user.add(buf_agent);
                });
            }
        }
        Ok(ForkResult::Child) => {
            drop(reader);
            let mut inbound_replies: Queue<[u8;5]> = queue![];
            
            let user_original = Arc::new(Mutex::new(inbound_replies));
            let user1 = user_original.clone();
            thread::spawn(move || {
                let mut buf_agent_reply = [0;5];
                loop{
                    let (number_of_bytes, _) = socket_external.recv_from(&mut buf_agent_reply)
                                                .expect("Didn't receive data");
                    if(<identify status message>){
                        /////extract address and status
                        //let mut server_address = 
                        //let mut stat = 
                        let mut idx = address_to_idx[server_address];
                        let mut buf = [0;2];
                        buf[0] = idx;
                        buf[1] = stat;
                        writer.write(&mut buf).unwrap();
                    }
                    else{
                        let mut locked_user = user1.lock().unwrap();
                        println!("{:?}", buf_agent_reply);


                        locked_user.add(buf_agent_reply);
                        println!("{:?}", locked_user.size());
                        drop(locked_user);
                    }
                }
            });
            let socket_internal = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            let user = user_original.clone();
            loop {
                // let (number_of_bytes, _) = socket_external.recv_from(&mut buf_agent_reply)
                                                // .expect("Didn't receive data");
                let mut locked_user = user.lock().unwrap();
                // println!("{:?}", locked_user.size());
                if locked_user.size()>0
                {
                    let mut buf_agent_reply = locked_user.peek().unwrap();
                    locked_user.remove();
                    drop(locked_user);
                    let client_port: u16 = ((buf_agent_reply[3] as u16) << 8) | (buf_agent_reply[4] as u16);
                    let socket_address_client_reply = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), client_port);
                    println!("hi{:?}", client_port);
                    let mut buf = [0;3];
                    for j in 0..3 {
                        buf[j]=buf_agent_reply[j];
                    }
                    println!("Arr: {:?}", buf);

                    socket_internal.send_to(&buf, socket_address_client_reply).expect("couldn't send data");
                }
                else
                {
                    drop(locked_user);
                    thread::sleep(time::Duration::from_millis(10));
                }
            }
        }
        Err(_) => {
            // Error
            println!("Error");
        }
    }
}