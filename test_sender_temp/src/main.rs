use std::thread;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use tokio::time::timeout; 

fn main() {
    let received = Arc::new(Mutex::new(0));
    for i in 0..500{
        let received1 = received.clone();
        thread::spawn(move || {
            let mut local_ctr = 0;
            for i in 0..1000{
                let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
                //socket.set_read_timeout(Some(std::time::Duration::from_millis(3000))).expect("set_read_timeout call failed");
                socket.send_to(&[1,2,3], "127.0.0.1:4245").expect("couldn't send data");
                // println!("port: {}",socket.local_addr().unwrap().port());
                let mut buf = [0;3];
                // receive timeout in socket
                // try and catch
                match socket.recv_from(&mut buf){
                    Ok((number_of_bytes, client_address)) => {
                        //println!("received: {:?}", buf);
                        local_ctr += 1;
                    },
                    Err(e) => {
                        //println!("error: {:?}", e);
                    }
                }
                //socket.recv_from(&mut buf).expect("Didn't receive data");
                local_ctr += 1;
            }
            
            let mut locked_user = received1.lock().unwrap();
            *locked_user += local_ctr;
            drop(locked_user);
            // println!("Arr: {:?}", buf);
        });
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let locked_user = received.lock().unwrap();
        println!("Received: {}", *locked_user);
        drop(locked_user);
        // println!("Received: {}", received);
    }
}



 // if timeout
                // catch! 
                // {
                //     try{
                //         socket.recv_from(&mut buf).expect("Didn't receive data");
                //     }
                //     catch error: io::Error
                //     {
                    
                //         println!("timeout");
                //         continue;
                //     }
                    
                // };