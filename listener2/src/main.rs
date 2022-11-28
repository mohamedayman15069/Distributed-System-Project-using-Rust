use timer::Timer;
use sysinfo::{CpuExt, System, SystemExt};
use std::net::UdpSocket;

// static mut socket:UdpSocket = UdpSocket::bind("127.0.0.1:4243").expect("couldn't bind to address");


fn get_cpu_util() -> i32{
	let mut sys = System::new();
    sys.refresh_cpu(); // Refreshing CPU information.
	let mut avg = 0;
	let mut count = 0;
	for cpu in sys.cpus() {
		avg += cpu.cpu_usage() as i32;
		count+=1;
		print!("{}% ", cpu.cpu_usage());
	}

	avg/=count;
	return avg;
}


fn main() {

	let mut socket = UdpSocket::bind("127.0.0.1:4243").expect("couldn't bind to address");
    let mut agents: Vec<String> = Vec::new();

	let SERVER_ADDRESSES:[&str;2] = ["127.0.0.1:4242","127.0.0.1:4244"];
	let MY_ADDRESS:&str = "127.0.0.1:4243";
	let mut ELECTION:[String;2] = [String::new(),String::new()];
	let mut RESULT:String = String::new();
	let mut SERVER_DOWN:String = String::new();

	let timer = Timer::new();

	// sechedule a task to run every 5 seconds 

	loop{	
		// Just do this loop for each 1 second using sleep ? 
		// sleep for one second 
		std::thread::sleep(std::time::Duration::from_secs(1));
		println!("Timer fired!");
		// start election
		println!("start election fn");
		//get cpu utilization
		let avg = get_cpu_util();

		// check that no election was already started by another server
		if ELECTION[0]=="" && ELECTION[1]==""{
			//send out my address and cpu utilization to both servers (right and left)
			let el = "e,".to_owned() + MY_ADDRESS + "," + avg.to_string().as_str();
			for addr in SERVER_ADDRESSES{
				socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			}	

		}
	//if election already started
		else{	
			let avg = get_cpu_util();
			//compare cpu utilizations with self
			let s1: i32 = ELECTION[0].split(',').last().expect("cannot get util from election").parse().unwrap();
			let s2: i32 = ELECTION[1].split(',').last().expect("cannot get util from election").parse().unwrap();

			let b_election = [ELECTION[0].as_bytes(),ELECTION[1].as_bytes()];
			if avg > s1 {
				if avg> s2 {
					//this server won - send new candidate election message to both servers
					println!("This server won");

					let el = "e,".to_owned() + MY_ADDRESS + "," + avg.to_string().as_str();

					for addr in SERVER_ADDRESSES{
						socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
					}				
				}else{
				//s2 won, send to the other server (not the one received from)
					println!("Server [0] won");
					socket.send_to(&b_election[1], SERVER_ADDRESSES[0]).expect("couldn't send data");
				}
			}else{
				//s1 won, send to the other server (not the one received from)
					println!("Server [1] won");
					socket.send_to(&b_election[0], SERVER_ADDRESSES[1]).expect("couldn't send data");
			}
			//check if both sides have same decision
			if ELECTION[0] == ELECTION[1]  && ELECTION[0] != "" {
				println!("Received 2 equal election messages");

				//send out result
				RESULT = "r,".to_owned() + ELECTION[0].split(',').nth(1).unwrap() + "," + ELECTION[0].split(',').nth(2).unwrap();
				println!("Sending election result: {}", RESULT);

				for addr in SERVER_ADDRESSES{
					let b_result = RESULT.as_bytes();
					socket.send_to(&b_result, addr).expect("couldn't send data");
				}
			}
		}

	loop {
		let mut buf = [0; 25];
		let (amt, src) = socket.recv_from(&mut buf).expect("Didn't receive data");
		println!("{} bytes from {}", amt, src);
		println!("data: {:?}", buf);

		// let msg = str::from_utf8(&buf).unwrap();
		let msg = String::from_utf8((&buf).to_vec()).unwrap();

		if SERVER_ADDRESSES.contains(&src.to_string().as_str()) { //if msg from another server
			if msg.split(',').nth(0).unwrap() == "r"{
				SERVER_DOWN = msg.split(',').nth(1).unwrap().to_string();
				if SERVER_DOWN == MY_ADDRESS{
					println!("IM GOING DOWN!");
					for agent in &agents{
						let down = "d,".to_owned()+ &SERVER_DOWN;
						let b_down = down.as_bytes();
						socket.send_to(&b_down, agent).expect("couldn't send data");
					}				
				}
				println!("{}",SERVER_DOWN.to_owned() + " IS DOWN!");

				ELECTION[0] = String::new();
				ELECTION[1] = String::new();
				RESULT.clear();
			}
			else if msg.split(',').nth(0).unwrap() == "e"{
				if src.to_string().as_str() == SERVER_ADDRESSES[0]{
					ELECTION[0] = msg;
				}else if src.to_string().as_str() == SERVER_ADDRESSES[1]{
					ELECTION[1] = msg;
				}
				let avg = get_cpu_util();
				//compare cpu utilizations with self
				let s1: i32 = ELECTION[0].split(',').last().expect("cannot get util from election").parse().unwrap();
				let s2: i32 = ELECTION[1].split(',').last().expect("cannot get util from election").parse().unwrap();

				let b_election = [ELECTION[0].as_bytes(),ELECTION[1].as_bytes()];
				if avg > s1 {
					if avg> s2 {
						//this server won - send new candidate election message to both servers
						println!("This server won");

						let el = "e,".to_owned() + MY_ADDRESS + "," + avg.to_string().as_str();

						for addr in SERVER_ADDRESSES{
							socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						}				
					}else{
					//s2 won, send to the other server (not the one received from)
						println!("Server [0] won");
						socket.send_to(&b_election[1], SERVER_ADDRESSES[0]).expect("couldn't send data");
					}
				}else{
					//s1 won, send to the other server (not the one received from)
						println!("Server [1] won");
						socket.send_to(&b_election[0], SERVER_ADDRESSES[1]).expect("couldn't send data");
				}
				//check if both sides have same decision
				if ELECTION[0] == ELECTION[1]  && ELECTION[0] != "" {
					println!("Received 2 equal election messages");

					//send out result
					RESULT = "r,".to_owned() + ELECTION[0].split(',').nth(1).unwrap() + "," + ELECTION[0].split(',').nth(2).unwrap();
					println!("Sending election result: {}", RESULT);

					for addr in SERVER_ADDRESSES{
						let b_result = RESULT.as_bytes();
						socket.send_to(&b_result, addr).expect("couldn't send data");
					}
				}
			}
		}
		//else: msg from agent
		else if !agents.contains(&src.to_string()) {
			agents.push(src.to_string());	//collecting list of agents in the system
		}
	

		println!("agents: {:?}", agents);
	}
}
// });
	
}