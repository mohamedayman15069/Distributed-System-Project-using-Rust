fn main() {
	use std::net::UdpSocket;

	let socket = UdpSocket::bind("127.0.0.1:4243").expect("couldn't bind to address");
    
    let mut buf = [0;5];
	loop{
		let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                                            .expect("Didn't receive data");

        // define a string that is the same length as the received data
        let filled_buf = &mut buf[..3];
        // print the received string
        println!("Arr: {:?}",filled_buf);
        // reverse the string
        filled_buf.reverse();
        // print the reversed string
        println!("Arr: {:?}",filled_buf);
        let mut full_buf = [0;5];
        for i in 0..3{
            full_buf[i] = filled_buf[i];
        }
        for i in 3..5{
            full_buf[i] = buf[i];
        }
        // send the reversed string back to the sender
        socket.send_to(& full_buf, src_addr).expect("couldn't send data");
    }
}
                                            