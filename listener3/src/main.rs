fn main() {
	use std::net::UdpSocket;

	let socket = UdpSocket::bind("127.0.0.1:4244").expect("couldn't bind to address");
    let mut buf = [0;3];
	loop{
        let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                                            .expect("Didn't receive data");
        let filled_buf = &mut buf[..number_of_bytes];
        println!("Arr: {:?}",filled_buf);
        filled_buf.reverse();
        println!("Arr: {:?}",filled_buf);
        socket.send_to(& filled_buf, src_addr).expect("couldn't send data");
    }
}

