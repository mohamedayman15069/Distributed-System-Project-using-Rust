use std::thread;
use std::collections::HashMap;

fn main() {
    //will change to external IP later
    let socket_external = UdpSocket::bind("127.0.0.1:12346").expect("couldn't bind to address");  
    
    let socket_internal = UdpSocket::bind("127.0.0.1:4245").expect("couldn't bind to address");

    let listener_addresses = ["127.0.0.1:4242", "127.0.0.1:4243", "127.0.0.1:4244"];
    let mut requests_map=HashMap::new();
    let i=0;
    loop{
        let mut buf = [0;3];
        let (number_of_bytes, sender_address) = socket_internal.recv_from(&mut buf)
                                                .expect("Didn't receive data");
        let temp = thread::spawn(move || {
            //we need to append port number of client to buf some way or another

            socket_external.send_to(&buf, listener_addresses[i%3].to_string()).expect("couldn't send data");
            i += 1;
            i %= 3;
            let mut buf = [0;4];
            let (number_of_bytes, _) = socket_external.recv_from(&mut buf)
                                            .expect("Didn't receive data");
            //We need to find a way to use the port number in buf
            //alongside localhost IP in order to use instead of placeholder below
            socket_internal.send_to(&buf, /*<placeholder>*/).expect("couldn't send data");
        });
        
    }
}
