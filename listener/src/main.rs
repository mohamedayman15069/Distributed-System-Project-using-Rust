use timer::Timer;
use sysinfo::{CpuExt, System, SystemExt};
use std::net::UdpSocket;
use std::sync::mpsc::channel;

// static mut socket:UdpSocket = UdpSocket::bind("127.0.0.1:4242").expect("couldn't bind to address");


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
#[derive(Clone)]
struct Election{
	ELECTION:[String;2],
	RESULT:String,
	SERVER_DOWN:String,
	MY_ADDRESS:&'static str,
	SERVER_ADDRESSES:[&'static str;2],
}

fn main() {

	let mut socket_ = UdpSocket::bind("127.0.0.1:4243").expect("couldn't bind to address");
	let mut socket = socket_.try_clone().expect("couldn't clone socket");
    let mut agents: Vec<String> = Vec::new();

	let SERVER_ADDRESSES:[&str;2] = ["127.0.0.1:4243","127.0.0.1:4244"];
	let MY_ADDRESS:&str = "127.0.0.1:4242";
	let mut ELECTION:[String;2] = [String::new(),String::new()];
	let mut RESULT:String = String::new();
	let mut SERVER_DOWN:String = String::new();
	let mut field = Election{ ELECTION:ELECTION,
						  RESULT:RESULT,
						  SERVER_DOWN:SERVER_DOWN,
						  MY_ADDRESS:MY_ADDRESS,
						  SERVER_ADDRESSES:SERVER_ADDRESSES};
	let timer = Timer::new();
	let (tx, rx) = channel();
	// pass in schedule_repeating a closure that takes a mutable reference to field
	// sechedule a task to run every 5 seconds
	let tx  = tx.clone(); 
	let handle = timer.schedule_repeating(chrono::Duration::milliseconds(5000), move || {
		
		// sleep for one second 
		println!("Timer fired!");
		// start election
		println!("start election fn");
		//get cpu utilization
		let avg = get_cpu_util();

		// check that no election was already started by another server
		if field.ELECTION[0]=="" && field.ELECTION[1]==""{
			//send out my address and cpu utilization to both servers (right and left)
			let el = "e,".to_owned() + field.MY_ADDRESS + "," + avg.to_string().as_str();
			for addr in field.SERVER_ADDRESSES{
				socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			}	

		}
	//if election already started
		else{	
			let avg = get_cpu_util();
			//compare cpu utilizations with self
			let s1: i32 = field.ELECTION[0].split(',').last().expect("cannot get util from election").parse().unwrap();
			let s2: i32 = field.ELECTION[1].split(',').last().expect("cannot get util from election").parse().unwrap();

			let b_election = [field.ELECTION[0].as_bytes(),field.ELECTION[1].as_bytes()];
			if avg > s1 {
				if avg> s2 {
					//this server won - send new candidate election message to both servers
					println!("This server won");

					let el = "e,".to_owned() + field.MY_ADDRESS + "," + avg.to_string().as_str();

					for addr in field.SERVER_ADDRESSES{
						socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
					}				
				}else{
				//s2 won, send to the other server (not the one received from)
					println!("Server [0] won");
					socket.send_to(&b_election[1], field.SERVER_ADDRESSES[0]).expect("couldn't send data");
				}
			}else{
				//s1 won, send to the other server (not the one received from)
					println!("Server [1] won");
					socket.send_to(&b_election[0], field.SERVER_ADDRESSES[1]).expect("couldn't send data");
			}
			//check if both sides have same decision
			if field.ELECTION[0] == field.ELECTION[1]  && field.ELECTION[0] != "" {
				println!("Received 2 equal election messages");

				//send out result
				field.RESULT = "r,".to_owned() + field.ELECTION[0].split(',').nth(1).unwrap() + "," + field.ELECTION[0].split(',').nth(2).unwrap();
				println!("Sending election result: {}", field.RESULT);

				for addr in SERVER_ADDRESSES{
					let b_result = field.RESULT.as_bytes();
					socket.send_to(&b_result, addr).expect("couldn't send data");
				}
			}
		}

		// Now, I will need to have a new variable of type Election
		let mut new_field = Election{ELECTION: field.ELECTION.clone(),
									RESULT: field.RESULT.clone(),
									SERVER_DOWN: field.SERVER_DOWN.clone(),
									MY_ADDRESS: field.MY_ADDRESS.clone(),
									SERVER_ADDRESSES: field.SERVER_ADDRESSES.clone()};

	
	let _ignored = tx.send((new_field, socket.try_clone())).unwrap(); // Avoid unwrapping here.

	});

	//(field,socket) = rx.recv().unwrap();

	loop {
		let (mut field,_) = rx.recv().unwrap(); 
		//let socket = socket_;
	 	let mut buf = [0; 25];
	 	let (amt, src) = socket_.recv_from(&mut buf).expect("Didn't receive data");
	 	println!("{} bytes from {}", amt, src);
	 	println!("data: {:?}", buf);

		let msg = std::str::from_utf8(&buf).unwrap();
	 	let msg = String::from_utf8((&buf).to_vec()).unwrap();

	 	if field.SERVER_ADDRESSES.contains(&src.to_string().as_str()) { //if msg from another server
	 		if msg.split(',').nth(0).unwrap() == "r"{
	 			field.SERVER_DOWN = msg.split(',').nth(1).unwrap().to_string();
	 			if field.SERVER_DOWN == field.MY_ADDRESS{
	 				println!("IM GOING DOWN!");
	 				for agent in &agents{
	 					let down = "d,".to_owned()+ &field.SERVER_DOWN;
	 					let b_down = down.as_bytes();
	 					socket_.send_to(&b_down, agent).expect("couldn't send data");
	 				}				
	 			}
	 			println!("{}",field.SERVER_DOWN.to_owned() + " IS DOWN!");

	 			field.ELECTION[0] = String::new();
	 			field.ELECTION[1] = String::new();
	 			field.RESULT.clear();
	 		}
	 		else if msg.split(',').nth(0).unwrap() == "e"{
	 			if src.to_string().as_str() == field.SERVER_ADDRESSES[0]{
	 				field.ELECTION[0] = msg;
	 			}else if src.to_string().as_str() == field.SERVER_ADDRESSES[1]{
	 				field.ELECTION[1] = msg;
	 			}
	 			let avg = get_cpu_util();
	 			//compare cpu utilizations with self
	 			let s1: i32 = field.ELECTION[0].split(',').last().expect("cannot get util from election").parse().unwrap();
	 			let s2: i32 = field.ELECTION[1].split(',').last().expect("cannot get util from election").parse().unwrap();

	 			let b_election = [field.ELECTION[0].as_bytes(),field.ELECTION[1].as_bytes()];
	 			if avg > s1 {
	 				if avg> s2 {
	 					//this server won - send new candidate election message to both servers
	 					println!("This server won");

	 					let el = "e,".to_owned() + field.MY_ADDRESS + "," + avg.to_string().as_str();

	 					for addr in field.SERVER_ADDRESSES{
	 						socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
	 					}				
	 				}else{
	 				//s2 won, send to the other server (not the one received from)
	 					println!("Server [0] won");
	 					socket_.send_to(&b_election[1], field.SERVER_ADDRESSES[0]).expect("couldn't send data");
	 				}
	 			}else{
	 				//s1 won, send to the other server (not the one received from)
	 					println!("Server [1] won");
	 					socket_.send_to(&b_election[0], field.SERVER_ADDRESSES[1]).expect("couldn't send data");
	 			}
	 			//check if both sides have same decision
	 			if field.ELECTION[0] == field.ELECTION[1]  && field.ELECTION[0] != "" {
	 				println!("Received 2 equal election messages");

	 				//send out result
	 				field.RESULT = "r,".to_owned() + field.ELECTION[0].split(',').nth(1).unwrap() + "," + field.ELECTION[0].split(',').nth(2).unwrap();
	 				println!("Sending election result: {}", field.RESULT);

	 				for addr in field.SERVER_ADDRESSES{
	 					let b_result = field.RESULT.as_bytes();
	 					socket_.send_to(&b_result, addr).expect("couldn't send data");
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