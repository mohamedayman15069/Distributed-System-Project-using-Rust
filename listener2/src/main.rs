fn main() {
	use std::net::UdpSocket;

	let socket = UdpSocket::bind("127.0.0.1:4243").expect("couldn't bind to address");
    
    let mut buf = [0;3];
	loop{
		let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                                            .expect("Didn't receive data");

        // define a string that is the same length as the received data
        let filled_buf = &mut buf[..number_of_bytes];
        // print the received string
        println!("Arr: {:?}",filled_buf);
        // reverse the string
        filled_buf.reverse();
        // print the reversed string
        println!("Arr: {:?}",filled_buf);
        // send the reversed string back to the sender
        socket.send_to(& filled_buf, src_addr).expect("couldn't send data");
    }
}
                                            