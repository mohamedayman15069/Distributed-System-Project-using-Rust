use timer::Timer;
use sysinfo::{CpuExt, System, SystemExt};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::thread::sleep;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::time::SystemTime;
use csv::Writer;
use std::error::Error;

#[derive(Clone)]
struct Election{	
	ELECTION:[String;2],
	RESULT:String,
	SERVER_DOWN:String,
	MY_ADDRESS:&'static str,
	SERVER_ADDRESSES:[&'static str;2],
	avg : i32
}

fn main() -> Result<(), Box<dyn Error>>{

	let MY_ADDRESS:&str = "127.0.0.1:4243";
	let random_seed = 42;
	let election_ms = 60000;
	let sleep_s = 15;
	let mut socket_ = UdpSocket::bind(MY_ADDRESS).expect("couldn't bind to address");
	let mut socket = socket_.try_clone().expect("couldn't clone socket");
    let mut agents: Vec<String> = Vec::new();

	let SERVER_ADDRESSES:[&str;2] = ["127.0.0.1:4242","127.0.0.1:4244"];
	let mut ELECTION:[String;2] = [String::new(),String::new()];
    let mut active = [1;2];

	let mut RESULT:String = String::new();
	let mut SERVER_DOWN:String = String::new();
	let mut field = Election{ ELECTION:ELECTION,
						  RESULT:RESULT,
						  SERVER_DOWN:SERVER_DOWN,
						  MY_ADDRESS:MY_ADDRESS,
						  SERVER_ADDRESSES:SERVER_ADDRESSES,
						  avg:0};
	let timer = Timer::new();
	// let d = SystemTime::now()
    // .duration_since(SystemTime::UNIX_EPOCH)
    // .expect("Duration since UNIX_EPOCH failed");
	let mut rng = StdRng::seed_from_u64(random_seed);
	let mut rng_ = rng.clone();
    let count = 100000+7;
	field.avg = (rng.gen::<i32>()%count +count)%(count);
	//println!("MY AVG: {}",avg);
	// pass in schedule_repeating a closure that takes a mutable reference to field
	// sechedule a task to run every 5 seconds
	let user_original = Arc::new(Mutex::new(field));
	let user1 = user_original.clone();
	
	let mut no_requests: u32 = 0;
	let user1_original_no_requests = Arc::new(Mutex::new(no_requests));

	let wtr = Arc::new(Mutex::new(Writer::from_path("number_of_requests.csv")?));
	let wtr1 = wtr.clone();
	
	let user1_no_requests = user1_original_no_requests.clone();
	let timer2 = Timer::new();
	let handle = timer2.schedule_repeating(chrono::Duration::milliseconds(10000), move || {
		let mut locked_user_no_requests = user1_no_requests.lock().unwrap();
		let mut locked_wtr = wtr1.lock().unwrap();
		locked_wtr.serialize(*locked_user_no_requests);
		locked_wtr.flush();
		drop(locked_wtr);
		println!("req_per_server: {:?}", locked_user_no_requests);
		drop(locked_user_no_requests);
	});

	let handle = timer.schedule_repeating(chrono::Duration::milliseconds(election_ms), move || {
		
		// sleep for one second 
		println!("Timer fired!");
		if active[0]==1 && active[1]==1 {
			let mut locked_user = user1.lock().unwrap();
			locked_user.avg = (rng_.gen::<i32>()%count +count)%(count);
			// if locked_user.ELECTION[0]=="" && locked_user.ELECTION[1]=="" {
				let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
				println!("STARTING ELECTION: {:?}", el);
				for addr in locked_user.SERVER_ADDRESSES{
					socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
				}	
			// }else{
			// 	println!("ELECTION ALREADY STARTED");
			// }
			drop(locked_user);
		}else{
			println!("ELECTION NOT STARTED - SERVER INACTIVE");
		}
	});

	let user2 = user_original.clone();
	let mut server_up = 1;
	let mut im_considered = 0;
	loop {
		// println!("Agents: {:?}", agents);
		//let socket_ = socket__;
	 	let mut buf = [0; 30];
		let	(amt, src) = socket_.recv_from(&mut buf).expect("Didn't receive data");
		
		//if flag 0 
		//dont reply
		//if flag 1
		//reply 

		if server_up == 0{
			continue;
		}
	 	// println!("\n{} bytes from {}", amt, src);
		let mut locked_user = user2.lock().unwrap();
	 	if locked_user.SERVER_ADDRESSES.contains(&src.to_string().as_str()) { //if msg from another server
			let mut msg = String::from_utf8((&buf).to_vec()).unwrap();
			// println!("msg: {}", msg);

			if amt==1 {
				let mut stat = 0;
				if buf[0] =='d' as u8 || buf[0]=='u' as u8
				{
					if buf[0]=='d' as u8
					{
						if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
							active[0] = 0;
						}
						else{
							active[1] = 0;
						}					
					}
					else if buf[0]=='u' as u8
					{    
						if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
							active[0] = 1;
						}
						else{
							active[1] = 1;
						}					
					}
				}
			}




			if msg.split(',').nth(0).unwrap() == "r"{
				if active[0] == 0 || active[1] == 0{
					continue;
				}
				if msg.split(',').nth(1).unwrap().to_string() == locked_user.SERVER_DOWN{
					println!("ELECTION DONE");
					locked_user.SERVER_DOWN = String::new();
				}
				else{
					locked_user.SERVER_DOWN = msg.split(',').nth(1).unwrap().to_string();
					if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
						println!("IM GOING DOWN!");
						let down = "d";
						let b_down = down.as_bytes();
						for agent in &agents{
							socket_.send_to(&b_down, agent).expect("couldn't send data");
						}
						for addr in locked_user.SERVER_ADDRESSES{
							socket_.send_to(&b_down, addr).expect("couldn't send data");
						}

						server_up = 0;
						sleep(Duration::from_secs(sleep_s));
						locked_user.avg = (rng.gen::<i32>()%count + count)%count;
						println!("MY AVG: {}",locked_user.avg);
						server_up = 1;
						println!("IM BACK UP!");
						
						let up = "u";
						let b_up = up.as_bytes();
						
						for agent in &agents{
							socket_.send_to(&b_up, agent).expect("couldn't send data");
						}			
						for addr in locked_user.SERVER_ADDRESSES{
							socket_.send_to(&b_up, addr).expect("couldn't send data");
						}		
					}
					// println!("{}",locked_user.SERVER_DOWN.to_owned() + " IS DOWN!");
				
					locked_user.ELECTION[0] = String::new();
					locked_user.ELECTION[1] = String::new();
					locked_user.RESULT.clear();
				}
	 		}
	 		else if msg.split(',').nth(0).unwrap() == "e"{
				//print avg

				let mut dest = "";
				println!("received e .. MY AVG : {}",locked_user.avg);
				if active[0] == 0 || active[1] == 0{
					continue;
				}
				// println!("msg: {:?}", msg);
				msg = msg.split('\0').nth(0).unwrap().to_string();
				// println!("msg: {:?}", msg);
	 			if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
	 				locked_user.ELECTION[0] = msg;
					dest = locked_user.SERVER_ADDRESSES[1];
					
	 			}else if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[1]{
	 				locked_user.ELECTION[1] = msg;
					dest = locked_user.SERVER_ADDRESSES[0];
	 			
				}


				if (locked_user.ELECTION[0]!="" && locked_user.ELECTION[0].split(',').nth(1).unwrap() == locked_user.MY_ADDRESS) || (locked_user.ELECTION[1]!="" && locked_user.ELECTION[1].split(',').nth(1).unwrap() == locked_user.MY_ADDRESS){
					im_considered = 1;
				}
				else{
					im_considered = 0;
				}
				
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

				//if i am in the options already
				if im_considered == 1{
					if locked_user.ELECTION[0]==""{
						//send elec1 
						let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
                        // println!("sending: {:?}", el);
						// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
						socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
					}
					else if locked_user.ELECTION[1]==""{
						//send elec0
						let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
                        // println!("sending: {:?}", el);
						// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
						socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
					//not empty and equal
					}else if locked_user.ELECTION[0].split(',').nth(1).unwrap() == locked_user.ELECTION[1].split(',').nth(1).unwrap(){
						println!("----------------------------------");
						// println!("Received 2 equal election messages");
		
						//send out result
						locked_user.RESULT = "r,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + locked_user.ELECTION[0].split(',').nth(2).unwrap();
						println!("SENDING RESULT: {}", locked_user.RESULT);

						locked_user.SERVER_DOWN = locked_user.ELECTION[0].split(',').nth(1).unwrap().to_string();
						if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
							println!("IM GOING DOWN!");
							let down = "d";//+ &locked_user.SERVER_DOWN;
							let b_down = down.as_bytes();
							for agent in &agents{

								socket_.send_to(&b_down, agent).expect("couldn't send data");
							}
							for server in &SERVER_ADDRESSES{
								socket_.send_to(&b_down, server).expect("couldn't send data");
							}
							server_up = 0;
							sleep(Duration::from_secs(sleep_s));
							locked_user.avg = (rng.gen::<i32>()%count +count)%(count);
							println!("MY AVG: {}",locked_user.avg);
							server_up = 1;
							println!("IM BACK UP!");
							locked_user.ELECTION[0] = String::new();
							locked_user.ELECTION[1] = String::new();
							let up = "u";
							let b_up = up.as_bytes();
							for agent in &agents{
								socket_.send_to(&b_up, agent).expect("couldn't send data");
							}
							for server in &SERVER_ADDRESSES{
								socket_.send_to(&b_up, server).expect("couldn't send data");
							}
						}
							
						for addr in locked_user.SERVER_ADDRESSES{
							let b_result = locked_user.RESULT.as_bytes();
							
							socket_.send_to(&b_result, addr).expect("couldn't send data");
						}
					}
					else{ //not equal and not empty
						//send max to not src
						if s1>s2{
							let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
							// println!("sending: {:?}", el);
							// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
						}else{
							let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
							// println!("sending: {:?}", el);
							// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
						}
					}

				//i am not one of the elected yet
				}else{
					if locked_user.ELECTION[0]==""{
						//compare elec1 with self
						if s2 > locked_user.avg{
							let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
							// println!("sending: {:?}", el);
							// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
						}
						else{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
							// println!("sending: {:?}", el);
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}	
						}
					}
					else if locked_user.ELECTION[1]==""{
						//compare elec0 with self
						if s1 > locked_user.avg{
							let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
							// println!("sending: {:?}", el);
							// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
							
						}
						else{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
							// println!("sending: {:?}", el);
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}
						}
					}else if locked_user.ELECTION[0].split(',').nth(1).unwrap() == locked_user.ELECTION[1].split(',').nth(1).unwrap(){
						if locked_user.avg > s1{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
							// println!("sending: {:?}", el);
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}
						}else{
							println!("----------------------------------");
							// println!("Received 2 equal election messages");
			
							//send out result
							locked_user.RESULT = "r,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + locked_user.ELECTION[0].split(',').nth(2).unwrap();
							// println!("SENDING RESULT: {}", locked_user.RESULT);

							locked_user.SERVER_DOWN = locked_user.ELECTION[0].split(',').nth(1).unwrap().to_string();
							if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
								println!("IM GOING DOWN!");
								let down = "d";//+ &locked_user.SERVER_DOWN;
								let b_down = down.as_bytes();
								for agent in &agents{

									socket_.send_to(&b_down, agent).expect("couldn't send data");
								}
								for server in &SERVER_ADDRESSES{
									socket_.send_to(&b_down, server).expect("couldn't send data");
								}
								server_up = 0;
								sleep(Duration::from_secs(sleep_s));
								locked_user.avg = (rng.gen::<i32>()%count +count)%(count);
								println!("MY AVG: {}",locked_user.avg);
								server_up = 1;
								println!("IM BACK UP!");
								locked_user.ELECTION[0] = String::new();
								locked_user.ELECTION[1] = String::new();
								let up = "u";
								let b_up = up.as_bytes();
								for agent in &agents{
									socket_.send_to(&b_up, agent).expect("couldn't send data");
								}
								for server in &SERVER_ADDRESSES{
									socket_.send_to(&b_up, server).expect("couldn't send data");
								}
							}
								
							for addr in locked_user.SERVER_ADDRESSES{
								let b_result = locked_user.RESULT.as_bytes();
								
								socket_.send_to(&b_result, addr).expect("couldn't send data");
							}
						}
					}
					else{ //not equal and not empty
						if s1 == s2 && locked_user.avg <= s1{
							//send last
							if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
								socket_.send_to(&locked_user.ELECTION[0].as_bytes(), dest).expect("couldn't send data");
							}
							else{
								// println!("sending: {:?}", locked_user.ELECTION[1]);
								// socket_.send_to(&locked_user.ELECTION[1].as_bytes(), locked_user.SERVER_ADDRESSES[0]).expect("couldn't send data");
								socket_.send_to(&locked_user.ELECTION[1].as_bytes(), dest).expect("couldn't send data");
							}
						}
						else if s1 > s2 && s1 > locked_user.avg{
							//send s1 
							let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
							// println!("sending: {:?}", el);
							// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");

						}else if s2 > s1 && s2 > locked_user.avg{
							//send s2
							let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
							// println!("sending: {:?}", el);
							// socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");

						}else if locked_user.avg > s1 && locked_user.avg > s2{
							//send avg
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
							// println!("sending: {:?}", el);
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}
						}
						else{
							println!("ERROR");
						}
					}
				}
			}





//////////// OLD CODE	
			// 	if locked_user.ELECTION[0] == "" && locked_user.ELECTION[1] == ""{
			// 		println!("INSIDE EMPTY ELECTION");
			// 		// println!("avg: {:?}", "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str());
			// 		// let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + avg.to_string().as_str();
			// 		// println!("sending: {:?}", el);
            //         // // println!("------------------el: {:?}", el);
			// 		// for addr in locked_user.SERVER_ADDRESSES{
			// 		// 	socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 		// }
			// 	}
			// 	else if locked_user.ELECTION[0] == ""{
			// 		if s2 > locked_user.avg{
			// 			let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
            //             println!("sending: {:?}", el);
			// 			// for addr in locked_user.SERVER_ADDRESSES{
			// 			// 	socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 			// }	
			// 			socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
			// 		}
			// 		else{
			// 			let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
            //             println!("sending: {:?}", el);
			// 			for addr in locked_user.SERVER_ADDRESSES{
			// 				socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 			}	
			// 		}
			// 	}
			// 	else if locked_user.ELECTION[1] == ""{
			// 		if s1 > locked_user.avg{
			// 			let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
            //             println!("sending: {:?}", el);
			// 			socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
						
			// 		}
			// 		else{
			// 			let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
            //             println!("sending: {:?}", el);
			// 			for addr in locked_user.SERVER_ADDRESSES{
			// 				socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 			}
			// 		}
			// 	}


			// 	//if s1 > s2
			// 		//if s1 > avg
			// 			//s1 won
			// 		//else 
			// 			//avg won
			// 	//else if s2 > avg
			// 		//s2 won
			// 	//else
			// 		//avg won


            //     else if locked_user.ELECTION[0] != locked_user.ELECTION[1] && locked_user.ELECTION[0] != "" && locked_user.ELECTION[1] != ""{
            //         println!("NOT EQUAL");
			// 		if s1 == s2{
			// 			if locked_user.avg <= s1{
			// 				//send last received message
			// 				// println!("sending: {:?}", msg);
			// 				if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
			// 					println!("sending: {:?}", locked_user.ELECTION[0]);
			// 					socket_.send_to(&locked_user.ELECTION[0].as_bytes(), locked_user.SERVER_ADDRESSES[1]).expect("couldn't send data");
			// 				}
			// 				else{
			// 					println!("sending: {:?}", locked_user.ELECTION[1]);
			// 					socket_.send_to(&locked_user.ELECTION[1].as_bytes(), locked_user.SERVER_ADDRESSES[0]).expect("couldn't send data");
			// 				}
			// 			}
			// 			else if locked_user.avg > s1{
			// 				//avg won
			// 				let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
			// 				println!("sending: {:?}", el);
			// 				for addr in locked_user.SERVER_ADDRESSES{
			// 					socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 				}	
			// 			} 
			// 		}
            //         else if s1 > s2{
            //             if s1 > locked_user.avg{
			// 				//s1 won
            //                 let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
            //                 println!("sending: {:?}", el);
            //                 // for addr in locked_user.SERVER_ADDRESSES{
            //                 //     socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
            //                 // }	
			// 				socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[1]).expect("couldn't send data");
							
            //             }
            //             else{
			// 				//avg won
			// 				let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
			// 				println!("sending: {:?}", el);
			// 				for addr in locked_user.SERVER_ADDRESSES{
			// 					socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 				}	
			// 			}
			// 		}
			// 		else if s2 > s1 && s2 > locked_user.avg{
			// 			//s2 won
			// 			let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
			// 			println!("sending: {:?}", el);
			// 			// for addr in locked_user.SERVER_ADDRESSES{
			// 			// 	socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 			// }
			// 			socket_.send_to(&el.as_bytes(), SERVER_ADDRESSES[0]).expect("couldn't send data");
			// 		}
			// 		else{
			// 			//avg won
			// 			let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.avg.to_string().as_str();
			// 			println!("sending: {:?}", el);
			// 			for addr in locked_user.SERVER_ADDRESSES{
			// 				socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
			// 			}
			// 		}
							
            //     }
			// 	//check if both sides have same decision
			// 	if locked_user.ELECTION[0] == locked_user.ELECTION[1]  && locked_user.ELECTION[0] != "" {
			// 		println!("----------------------------------");
			// 		println!("Received 2 equal election messages");
	
			// 		//send out result
			// 		locked_user.RESULT = "r,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + locked_user.ELECTION[0].split(',').nth(2).unwrap();
			// 		println!("SENDING RESULT: {}", locked_user.RESULT);



			// 		//TESTING
			// 		locked_user.SERVER_DOWN = locked_user.ELECTION[0].split(',').nth(1).unwrap().to_string();
			// 		if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
			// 			println!("IM GOING DOWN!");
			// 			let down = "d";//+ &locked_user.SERVER_DOWN;
			// 			let b_down = down.as_bytes();
			// 			for agent in &agents{

			// 				socket_.send_to(&b_down, agent).expect("couldn't send data");
			// 			}
			// 			for server in &SERVER_ADDRESSES{
			// 				socket_.send_to(&b_down, server).expect("couldn't send data");
			// 			}
			// 			server_up = 0;
			// 			sleep(Duration::from_secs(sleep_s));
			// 			locked_user.avg = (rng.gen::<i32>()%count +count)%(count);
			// 			println!("MY AVG: {}",locked_user.avg);
			// 			server_up = 1;
			// 			println!("IM BACK UP!");
			// 			let up = "u";
			// 			let b_up = up.as_bytes();
			// 			for agent in &agents{
			// 				socket_.send_to(&b_up, agent).expect("couldn't send data");
			// 			}
			// 			for server in &SERVER_ADDRESSES{
			// 				socket_.send_to(&b_up, server).expect("couldn't send data");
			// 			}
			// 		}
						
			// 		for addr in locked_user.SERVER_ADDRESSES{
			// 			let b_result = locked_user.RESULT.as_bytes();
						
			// 			socket_.send_to(&b_result, addr).expect("couldn't send data");
			// 		}
			// 	}
			// }


/////////////OLD CODE END



		}
	 	//else: msg from agent
	 	else{ 
			let user2_no_requests = user1_original_no_requests.clone();
			let mut locked_user = user2_no_requests.lock().unwrap();
			*locked_user += 1;
			drop(locked_user);
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
				// println!("received: {:?}", local_buf);
				socket_request.send_to(&local_buf, src).expect("couldn't send data");
			});
		}	
		 drop(locked_user);

	 	// println!("agents: {:?}", agents);
		}
}