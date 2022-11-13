use std::thread;
use std::net::UdpSocket;
fn main() {
    for i in 0..10 {
        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            socket.send_to(&[1,2,3], "127.0.0.1:4245").expect("couldn't send data");
            println!("port: {}",socket.local_addr().unwrap().port());
            let mut buf = [0;3];
            socket.recv_from(&mut buf).expect("Didn't receive data");
            println!("Arr: {:?}", buf);
        });
    }
    loop {}
}