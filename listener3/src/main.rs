use timer::Timer;
use sysinfo::{CpuExt, System, SystemExt};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::thread::sleep;


fn get_cpu_util() -> i32{
	let mut sys = System::new();
    sys.refresh_cpu(); // Refreshing CPU information.
	let mut avg = 2500;
	let mut count = 0;
	for cpu in sys.cpus() {
		avg += cpu.cpu_usage() as i32;
		count+=1;
		print!("{}% ", cpu.cpu_usage());
	}
	println!("");

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

	let mut socket_ = UdpSocket::bind("127.0.0.1:4244").expect("couldn't bind to address");
	let mut socket = socket_.try_clone().expect("couldn't clone socket");
    let mut agents: Vec<String> = Vec::new();

	let SERVER_ADDRESSES:[&str;2] = ["127.0.0.1:4242","127.0.0.1:4243"];
	let MY_ADDRESS:&str = "127.0.0.1:4244";
	let mut ELECTION:[String;2] = [String::new(),String::new()];
	let mut RESULT:String = String::new();
	let mut SERVER_DOWN:String = String::new();
	let mut field = Election{ ELECTION:ELECTION,
						  RESULT:RESULT,
						  SERVER_DOWN:SERVER_DOWN,
						  MY_ADDRESS:MY_ADDRESS,
						  SERVER_ADDRESSES:SERVER_ADDRESSES};
	let timer = Timer::new();
	// pass in schedule_repeating a closure that takes a mutable reference to field
	// sechedule a task to run every 5 seconds
	let user_original = Arc::new(Mutex::new(field));
	let user1 = user_original.clone();
	let handle = timer.schedule_repeating(chrono::Duration::milliseconds(10000), move || {
		
		// sleep for one second 
		println!("Timer fired!");
		// start election
		// println!("start election fn");
		//get cpu utilization
		let avg = get_cpu_util();

		// check that no election was already started by another server
		// println!("check that no election was already started by another server");
		let mut locked_user = user1.lock().unwrap();
		// println!("field.ELECTION[1] = {}",locked_user.ELECTION[1]);
		if locked_user.ELECTION[0]=="" && locked_user.ELECTION[1]=="" {
			//send out my address and cpu utilization to both servers (right and left)
			// println!("avg: {:?}", "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str());
			let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
            println!("STARTING ELECTION: {:?}", el);
			// println!("------------------el: {:?}", el);
			for addr in locked_user.SERVER_ADDRESSES{
				socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			}	
		}

		drop(locked_user);
	});

	let user2 = user_original.clone();
	let mut server_up = 1;
	loop {
		println!("Agents: {:?}", agents);
		//let socket_ = socket__;
	 	let mut buf = [0; 30];
		// let mut amt = 0;
		// let mut src: std::net::SocketAddr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0);
		

		let	(amt, src) = socket_.recv_from(&mut buf).expect("Didn't receive data");
		
		//if flag 0 
		//dont reply
		//if flag 1
		//reply 

		if server_up == 0{
			continue;
		}

		// while server_up==0
	 	// {
		// 	(amt, src) = socket_.recv_from(&mut buf).expect("Didn't receive data");
		// }
		// (amt, src) = socket_.recv_from(&mut buf).expect("Didn't receive data");
	 	println!("\n{} bytes from {}", amt, src);
	 	// println!("data: {:?}", buf);

		// let msg = std::str::from_utf8(&buf).unwrap();
	 	

		let mut locked_user = user2.lock().unwrap();

	 	if locked_user.SERVER_ADDRESSES.contains(&src.to_string().as_str()) { //if msg from another server
			let mut msg = String::from_utf8((&buf).to_vec()).unwrap();
			println!("msg: {}", msg);
			if msg.split(',').nth(0).unwrap() == "r"{
				if msg.split(',').nth(1).unwrap().to_string() == locked_user.SERVER_DOWN{
					//print election done
					println!("ELECTION DONE");
					locked_user.SERVER_DOWN = String::new();
				}
				else{
					locked_user.SERVER_DOWN = msg.split(',').nth(1).unwrap().to_string();
					if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
						println!("IM GOING DOWN!");
						for agent in &agents{
							let down = "d";//.to_owned()+ &locked_user.SERVER_DOWN;
							let b_down = down.as_bytes();
							socket_.send_to(&b_down, agent).expect("couldn't send data");
						}
						server_up = 0;
						sleep(Duration::from_secs(3));
						server_up = 1;
						println!("IM BACK UP!");
						for agent in &agents{
							let up = "u";//+ &locked_user.SERVER_DOWN;
							let b_up = up.as_bytes();
							socket_.send_to(&b_up, agent).expect("couldn't send data");
						}					
					}
					println!("{}",locked_user.SERVER_DOWN.to_owned() + " IS DOWN!");
				
					locked_user.ELECTION[0] = String::new();
					locked_user.ELECTION[1] = String::new();
					locked_user.RESULT.clear();
				}
	 		}
	 		else if msg.split(',').nth(0).unwrap() == "e"{
					
				// println!("msg: {:?}", msg);
				msg = msg.split('\0').nth(0).unwrap().to_string();
				println!("msg: {:?}", msg);
	 			if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
	 				locked_user.ELECTION[0] = msg;
	 			}else if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[1]{
	 				locked_user.ELECTION[1] = msg;
				}
				
				let avg = get_cpu_util();
				let mut s1:i32 = -1; let mut s2:i32 = -1;
				if locked_user.ELECTION[0] !=""
				{
					// s1 = locked_user.ELECTION[0].split(',').last().expect("cannot get util from election").split('\0').next().expect("cannot get util from election").parse::<i32>().unwrap();
					s1 = locked_user.ELECTION[0].split(',').last().expect("cannot get util from election").parse::<i32>().unwrap();
				}
				if locked_user.ELECTION[1] != ""
				{
					// s2 = locked_user.ELECTION[1].split(',').last().expect("cannot get util from election").split('\0').next().expect("cannot get util from election").parse::<i32>().unwrap();
					s2 = locked_user.ELECTION[1].split(',').last().expect("cannot get util from election").parse::<i32>().unwrap();
				}
				let b_election = [locked_user.ELECTION[0].as_bytes(),locked_user.ELECTION[1].as_bytes()];	
	
                //print election
                println!("ELECTION: {:?}", locked_user.ELECTION);

				//if elec[0] is empty
					//compare elec 1 with self
				//else if elec[1] is empty
					//compare elec 0 with self
				//else compare all 3
	
				//if elec[0] and elec[1] are empty
					//send out my address and cpu utilization to both servers (right and left)
	
				
				if locked_user.ELECTION[0] == "" && locked_user.ELECTION[1] == ""{
					// println!("avg: {:?}", "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str());
					let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
					println!("sending: {:?}", el);
                    // println!("------------------el: {:?}", el);
					for addr in locked_user.SERVER_ADDRESSES{
						socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
					}
				}
				else if locked_user.ELECTION[0] == ""{
					if s2 > avg{
						let el = "e,".to_owned() + locked_user.SERVER_ADDRESSES[1] + "," + s2.to_string().as_str();
                        println!("sending: {:?}", el);
						// for addr in locked_user.SERVER_ADDRESSES{
						// 	socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						// }	
						socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
					}
					else{
						let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
                        println!("sending: {:?}", el);
						for addr in locked_user.SERVER_ADDRESSES{
							socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						}	
					}
				}
				else if locked_user.ELECTION[1] == ""{
					if s1 > avg{
						let el = "e,".to_owned() + locked_user.SERVER_ADDRESSES[0] + "," + s1.to_string().as_str();
                        println!("sending: {:?}", el);

						// for addr in locked_user.SERVER_ADDRESSES{
						// 	socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						// }	
						socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
						
					}
					else{
						let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
                        println!("sending: {:?}", el);
						for addr in locked_user.SERVER_ADDRESSES{
							socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						}
					}
				}


				//if s1 > s2
					//if s1 > avg
						//s1 won
					//else 
						//avg won
				//else if s2 > avg
					//s2 won
				//else
					//avg won


                else if locked_user.ELECTION[0] != locked_user.ELECTION[1]{
                    println!("NOT EQUAL");
                    if s1 > s2{
                        if s1 > avg{
							//s1 won
                            let el = "e,".to_owned() + locked_user.SERVER_ADDRESSES[0] + "," + s1.to_string().as_str();
                            println!("sending: {:?}", el);
                            // for addr in locked_user.SERVER_ADDRESSES{
                            //     socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
                            // }	
							socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
							
                        }
                        else{
							//avg won
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
							println!("sending: {:?}", el);
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}	
						}
					}
					else if s2 > avg{
						//s2 won
						let el = "e,".to_owned() + locked_user.SERVER_ADDRESSES[1] + "," + s2.to_string().as_str();
						println!("sending: {:?}", el);
						// for addr in locked_user.SERVER_ADDRESSES{
						// 	socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						// }
						socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
					}
					else{
						//avg won
						let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
						println!("sending: {:?}", el);
						for addr in locked_user.SERVER_ADDRESSES{
							socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
						}
					}
							
                }
				//check if both sides have same decision
				if locked_user.ELECTION[0] == locked_user.ELECTION[1]  && locked_user.ELECTION[0] != "" {
					println!("----------------------------------");
					println!("Received 2 equal election messages");
	
					//send out result
					locked_user.RESULT = "r,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + locked_user.ELECTION[0].split(',').nth(2).unwrap();
					println!("SENDING RESULT: {}", locked_user.RESULT);



					//TESTING
					locked_user.SERVER_DOWN = locked_user.ELECTION[0].split(',').nth(1).unwrap().to_string();
					if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
						println!("IM GOING DOWN!");
						for agent in &agents{
							let down = "d";//+ &locked_user.SERVER_DOWN;
							let b_down = down.as_bytes();
							socket_.send_to(&b_down, agent).expect("couldn't send data");
						}
						server_up = 0;
						sleep(Duration::from_secs(3));
						server_up = 1;
						println!("IM BACK UP!");
						for agent in &agents{
							let up = "u";//+ &locked_user.SERVER_DOWN;
							let b_up = up.as_bytes();
							socket_.send_to(&b_up, agent).expect("couldn't send data");
						}					
					}



					
					for addr in locked_user.SERVER_ADDRESSES{
						let b_result = locked_user.RESULT.as_bytes();
						
						socket_.send_to(&b_result, addr).expect("couldn't send data");
					}
				}
			}
		}
	 	//else: msg from agent
	 	else{ 
			if !agents.contains(&src.to_string()) {

	 			agents.push(src.to_string());	//collecting list of agents in the system
	 	 	}
		
			let socket_request = socket_.try_clone().expect("couldn't clone socket");
			let mut local_buf = [0;5];
			for i in 0..5{
				local_buf[i] = buf[i];
			}
			thread::spawn(move ||{
				let mut data_buf = [0;3];
				for i in 0..3{
					data_buf[i] = local_buf[i];
				}
				data_buf.reverse();
				for i in 0..3{
					local_buf[i] = data_buf[i];
				}
				println!("received: {:?}", local_buf);
				socket_request.send_to(&local_buf, src).expect("couldn't send data");
			});
		}	
		 drop(locked_user);

	 	// println!("agents: {:?}", agents);
		}
}