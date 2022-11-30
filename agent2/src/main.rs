use std::{thread, time};
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use nix::unistd::{fork, ForkResult};
use queues::*;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;
use std::collections::HashMap;
use timer::Timer;
use csv::Writer;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize)]
struct request_distribution{
    server_0: u128,
    server_1: u128,
    server_2: u128,
}

fn main()  -> Result<(), Box<dyn Error>>{
    // Socket that commuincates with servers
    let socket_external = UdpSocket::bind("10.7.57.94:0").expect("couldn't bind to address");
    // Active status for servers (0: down, 1: up)
    let mut active = [1;3];
    let active_mutex = Arc::new(Mutex::new(active));
    let server_addresses = ["10.7.57.247:4242", "10.7.57.254:4242", "10.7.57.232:4242"];
    // Map that maps server addresses to their index in the server_addresses array
    let mut address_to_idx = HashMap::new();
    // Populate map
    for i in 0..3 {
        address_to_idx.insert(server_addresses[i as usize], i);
    }
    // Pipe for comunicating server status between both processes
    let (mut reader, mut writer) = os_pipe::pipe().unwrap();
    match unsafe{fork()} {
        // Parent handles requests from clients and forwards them to servers
        Ok(ForkResult::Parent { child, .. }) => {
            drop(writer);
            
            let mut req_per_server: [u128;3] = [0;3];
            let req_p_server_mutex = Arc::new(Mutex::new(req_per_server));

            let mut outbound_requests: Queue<[u8;5]> = queue![];
            
            let outbound_requests_mutex = Arc::new(Mutex::new(outbound_requests));
            
            
            
            let mut i=0;
            let user1_outbound_requests = outbound_requests_mutex.clone();
            let user1_req_p_server = req_p_server_mutex.clone();
            let user1_active = active_mutex.clone();
            
            // This thread gets requests from queue and sends them to servers
            thread::spawn(move || {
                loop{
                    let mut locked_user_outbound_requests = user1_outbound_requests.lock().unwrap();
                    if locked_user_outbound_requests.size()>0
                    {
                        let temp: [u8;5] = locked_user_outbound_requests.peek().unwrap();
                        locked_user_outbound_requests.remove();
                        drop(locked_user_outbound_requests);
                        
                        let mut locked_user_active = user1_active.lock().unwrap();
                        while locked_user_active[i] == 0
                        {
                            i += 1;
                            i %= 3;
                        }
                        drop(locked_user_active);
                        let mut locked_user_req_p_server = user1_req_p_server.lock().unwrap();
                        locked_user_req_p_server[i] += 1;
                        drop(locked_user_req_p_server);
                        socket_external.send_to(&temp, server_addresses[i]).expect("couldn't send data");
                        i += 1;
                        i %= 3;
                    }
                    else
                    {
                        drop(locked_user_outbound_requests);
                        thread::sleep(time::Duration::from_millis(10));
                    }
                }
            });
            // Writer that writes distribution of requests on servers
            let wtr = Arc::new(Mutex::new(Writer::from_path("request_distribution.csv")?));
            let wtr_clone = wtr.clone();
            
            let user2_req_p_server = req_p_server_mutex.clone();
            let timer = Timer::new();
            let handle = timer.schedule_repeating(chrono::Duration::milliseconds(10000), move || {
                let mut locked_user_req_p_server = user2_req_p_server.lock().unwrap();
                let mut locked_wtr = wtr_clone.lock().unwrap();
                let mut request_distribution_local = request_distribution{
                    server_0: locked_user_req_p_server[0],
                    server_1: locked_user_req_p_server[1],
                    server_2: locked_user_req_p_server[2],
                };
                locked_wtr.serialize(request_distribution_local);
                locked_wtr.flush();
                drop(locked_wtr);
                println!("req_per_server: {:?}", locked_user_req_p_server);
                drop(locked_user_req_p_server);
            });
            // Update active status of servers
            let user2_active = active_mutex.clone();
            thread::spawn(move || {
                loop{
                    let mut buf = [0;2];   // server idx, status
                    reader.read(&mut buf).unwrap();
                    
                    let mut locked_user_active = user2_active.lock().unwrap();
                    locked_user_active[buf[0] as usize] = buf[1];
                    println!("Active: {:?}", locked_user_active);
                    drop(locked_user_active);
                }
            });
            // Socket for receiving requests from clients
            let socket_internal = UdpSocket::bind("127.0.0.1:4245").expect("couldn't bind to address");

            // Receive requests from clients
            loop{
                let mut buf = [0;3];
                let (number_of_bytes, client_address) = socket_internal.recv_from(&mut buf)
                                                        .expect("Didn't receive data");
                let user = outbound_requests_mutex.clone();
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
                    drop(locked_user);
                });
            }
        }
        // Child receives responses/active status from servers and sends replies back to clients
        Ok(ForkResult::Child) => {
            drop(reader);
            // Holds replies before sending back to clients
            let mut inbound_replies: Queue<[u8;5]> = queue![];
            
            let outbound_requests_mutex = Arc::new(Mutex::new(inbound_replies));
            let user1_outbound_requests = outbound_requests_mutex.clone();
            thread::spawn(move || {
                let mut buf_agent_reply = [0;5];
                loop{
                    let (number_of_bytes, server_address) = socket_external.recv_from(&mut buf_agent_reply)
                                                .expect("Didn't receive data");
                    // Updating active status of servers
                    if number_of_bytes==1 {
                        let mut stat = 0;
                        if buf_agent_reply[0] =='d' as u8 || buf_agent_reply[0]=='u' as u8
                        {
                            if buf_agent_reply[0]=='d' as u8
                            {
                                stat = 0;
                            }
                            else if buf_agent_reply[0]=='u' as u8
                            {    
                                stat = 1;
                            }
                            println!("server: {:?}, stat: {:?}", server_address, stat);
                            let mut idx = address_to_idx[&server_address.to_string().as_str()];
                            let mut buf = [0;2];
                            buf[0] = idx;
                            buf[1] = stat;
                            writer.write(&mut buf).unwrap();
                        }
                    }
                    // Push reply to queue
                    else{
                        let mut locked_user = user1_outbound_requests.lock().unwrap();

                        locked_user.add(buf_agent_reply);
                        drop(locked_user);
                    }
                }
            });
            let socket_internal = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            let user = outbound_requests_mutex.clone();
            // Send replies from queue to respective clients
            loop {
                let mut locked_user = user.lock().unwrap();
                if locked_user.size()>0
                {
                    let mut buf_agent_reply = locked_user.peek().unwrap();
                    locked_user.remove();
                    drop(locked_user);
                    let client_port: u16 = ((buf_agent_reply[3] as u16) << 8) | (buf_agent_reply[4] as u16);
                    let socket_address_client_reply = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), client_port);

                    let mut buf = [0;3];
                    for j in 0..3 {
                        buf[j]=buf_agent_reply[j];
                    }

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
    Ok(())
}