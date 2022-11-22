// use std::time::Instant;
// use std::time::Duration;
// use std::time::SystemTime;
// use sysinfo::{CpuExt, System, SystemExt};
// use sys_metrics::{cpu::*};
// use std::time::Timer;
use timer::Timer;
use sysinfo::{CpuExt, System, SystemExt};

// extern crate cpu;
// use cpu::Cpu;

let serverAddresses = vec!["127.0.0.1:4243","127.0.0.1:4244"];
let myAddress = "127.0.0.1:4242";
let mut election = vec!["",""];
let mut result = "";
let mut serverDown = "";
fn getCPUutil() -> i32{
	let mut sys = System::new();
    sys.refresh_cpu(); // Refreshing CPU information.
	let mut avg = 0;
	let mut count = 0;
	for cpu in sys.cpus() {
		avg +=cpu.cpu_usage();
		count+=1;
		print!("{}% ", cpu.cpu_usage());
	}

	avg/=count;
	return avg;
}
fn startelection(socket: UdpSocket){

//message formats
	//election message = "e,addr,util"
	//result message = "r,addr,util"


	println!("start election fn");
//get cpu utilization
	// let mut sys = System::new();
    // sys.refresh_cpu(); // Refreshing CPU information.
	// let mut avg = 0;
	// let mut count = 0;
	// for cpu in sys.cpus() {
	// 	avg +=cpu.cpu_usage();
	// 	count++;
	// 	print!("{}% ", cpu.cpu_usage());
	// }

	// avg/=count;
	let avg = getCPUutil();

// check that no election was already started by another server
	if election[0]=="" && election[1]==""{
		//send out my address and cpu utilization to both servers (right and left)
		let el = "e," + myAddress + "," + avg.to_string();
		for addr in serverAddresses{
			socket.send_to(&el, addr).expect("couldn't send data");
		}	

	}
//if election already started
	else{	

		checkElection();
	// 	//compare cpu utilizations with self
	// 	let mut s1: i32 = election[0].split(',').last().parse().unwrap();
	// 	let mut s2: i32 = election[1].split(',').last().parse().unwrap();

				
	// 	if avg >s1 {
	// 		if avg>s2 {
	// 			//this server won - send new candidate election message to both servers
	// 			let el = "e," + myAddress + "," + avg.to_string();
	// 			for addr in serverAddresses{
	// 				//send string el 
	// 				socket.send_to(&el, addr).expect("couldn't send data");
	// 			}				
	// 		}else{
	// 		//s2 won, send to the other server (not the one received from)
	// 			socket.send_to(&s2, serverAddresses[0]).expect("couldn't send data");
	// 		}
	// 	}else{
	// 		//s1 won, send to the other server (not the one received from)
	// 		socket.send_to(&s1, serverAddresses[1]).expect("couldn't send data");
	// 	}
	// //check if both sides have same decision
	// 	else if s1 == s2 {
	// 		//send out result
	// 		result = "r," + s1.split(',')[1] + "," + s1.split(',')[2];
	// 		for addr in serverAddresses{
	// 			socket.send_to(&result.to_bytes(), addr).expect("couldn't send data");
	// 		}
	// 	}
	}

	//reset election vector 
	// election = ["",""];
}

fn checkElection(){
	//compare cpu utilizations with self
	let mut s1: i32 = election[0].split(',').last().parse().unwrap();
	let mut s2: i32 = election[1].split(',').last().parse().unwrap();

			
	if avg > s1 {
		if avg> s2 {
			//this server won - send new candidate election message to both servers
			println!("This server won");

			let el = "e," + myAddress + "," + avg.to_string();
			for addr in serverAddresses{
				socket.send_to(&el.to_bytes(), addr).expect("couldn't send data");
			}				
		}else{
		//s2 won, send to the other server (not the one received from)
			println!("Server [0] won");
			socket.send_to(&election[1].to_bytes(), serverAddresses[0]).expect("couldn't send data");
		}
	}else{
		//s1 won, send to the other server (not the one received from)
			println!("Server [1] won");
			socket.send_to(&election[0].to_bytes(), serverAddresses[1]).expect("couldn't send data");
	}
	//check if both sides have same decision
	if s1 == s2  && s1 != "" {
		println!("Received 2 equal election messages");

		//send out result
		result = "r," + s1.split(',')[1] + "," + s1.split(',')[2];
		println!("Sending election result: " + result);

		for addr in serverAddresses{
			socket.send_to(&result.to_bytes(), addr).expect("couldn't send data");
		}
	}

}

fn main() {
    use std::net::UdpSocket;

	let socket = UdpSocket::bind("127.0.0.1:4242").expect("couldn't bind to address");
	// let serverAddresses = vec!["127.0.0.1:4243","127.0.0.1:4244"];
    // let mut buf = [0;3];
    let mut agents: Vec<String> = Vec::new();

	let timer = Timer::new();

    let _timer_handle = timer.schedule_repeating(chrono::Duration::milliseconds(10000), || {
		println!("Timer fired!");
		startelection(socket);
	});


	loop {
    	let mut buf = [0; 25];
    	let (amt, src) = socket.recv_from(&mut buf).expect("Didn't receive data");
    	println!("{} bytes from {}", amt, src);
    	println!("data: {:?}", buf);

		let msg = buf.to_string();
		if serverAddresses.contains(&src) { //if msg from another server
			if msg[0] == 'r'{
				serverDown = msg.split(',')[1];
				if serverDown == myAddress{
					println!("IM GOING DOWN!");
					for agent in agents{
						let down = "d,"+serverDown;
						socket.send_to(&down.to_bytes(), addr).expect("couldn't send data");
					}				
				}
				println!(serverDown + " IS DOWN!");

				election[0] = "";
				election[1] = "";
				result = "";
			}
			else if msg[0]=='e'{
				if src == serverAddresses[0]{
					election[0] = msg;
				}else if src == serverAddresses[1]{
					election[1] = msg;
				}
				checkElection();
			}
		}
		//else: msg from agent
		else if !agents.contains(&src.to_string()) {
    		agents.push(src.to_string());	//collecting list of agents in the system
		}
		

    	println!("agents: {:?}", agents);
    }
}