use std::thread;
use std::collections::HashMap;

fn main() {
    let socket_external = UdpSocket::bind("127.0.0.1:12346").expect("couldn't bind to address");
    let socket_internal = UdpSocket::bind("127.0.0.1:4245").expect("couldn't bind to address");

    let listener_addresses = ["127.0.0.1:4242", "127.0.0.1:4243", "127.0.0.1:4244"];
    let random_arrays = [[1,2,3], [4,5,6], [7,8,9]];
    let mut requests_map=HashMap::new();
    let i=0;
    loop{
        let mut buf = [0,[0;3]];
        let (number_of_bytes, sender_address) = socket_internal.recv_from(&mut buf)
                                                .expect("Didn't receive data");
        requests_map.insert(buf[0], sender_address);
        let temp = thread::spawn(move || {
            socket_external.send_to(&random_array[i%3], listener_address[i%3].to_string()).expect("couldn't send data");
            let mut buf = [0,[0;3]];
            let (number_of_bytes, _) = socket_external.recv_from(&mut buf)
                                            .expect("Didn't receive data");
            socket_internal.send_to(&buf, requests_map[buf[0]].to_string()).expect("couldn't send data");
        });
        
    }
}
