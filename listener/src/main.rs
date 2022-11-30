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
	AVG : i32
}

fn main() -> Result<(), Box<dyn Error>>{

	let MY_ADDRESS:&str = "10.7.57.247:4242";
	let random_seed = 41;
	let election_ms = 60000;
	let sleep_s = 15;

	const MOD :i32 = 100000+7;
	let mut rng = StdRng::seed_from_u64(random_seed);
	let mut cloned_rng = rng.clone();
	let avg = (rng.gen::<i32>()%MOD +MOD)%(MOD);

	// Add the needed variables in a struct
	let SERVER_ADDRESSES:[&str;2] = ["10.7.57.254:4242","10.7.57.232:4242"];
	let mut ELECTION:[String;2] = [String::new(),String::new()];
	let mut RESULT:String = String::new();
	let mut SERVER_DOWN:String = String::new();
	let mut election_Obj = Election{ ELECTION:ELECTION,
						  RESULT:RESULT,
						  SERVER_DOWN:SERVER_DOWN,
						  MY_ADDRESS:MY_ADDRESS,
						  SERVER_ADDRESSES:SERVER_ADDRESSES,
						  AVG:avg};
	

	let timer = Timer::new();
	let timer2 = Timer::new();
    let mut active = [1;2];
	let mut no_requests: u32 = 0;
	let mut server_up = 1;
	let mut im_considered = 0;
	

	//Bind with a socket
	let mut socket_ = UdpSocket::bind(MY_ADDRESS).expect("couldn't bind to address");
	let mut socket = socket_.try_clone().expect("couldn't clone socket");
    let mut agents: Vec<String> = Vec::new();



	let locked_election = Arc::new(Mutex::new(election_Obj));
	let user1_original_no_requests = Arc::new(Mutex::new(no_requests));
	let wtr = Arc::new(Mutex::new(Writer::from_path("number_of_requests.csv")?));

	// Cloning the needed variables to pass to the threads
	let user1 = locked_election.clone();
	let user2 = locked_election.clone();
	let wtr1 = wtr.clone();
	let user1_no_requests = user1_original_no_requests.clone();


	let election_scheduler = timer.schedule_repeating(chrono::Duration::milliseconds(election_ms), move || {
		
		// sleep for one second 
		println!("Timer fired!");
		if active[0]==1 && active[1]==1 {
			let mut locked_user = user1.lock().unwrap();
			locked_user.AVG = (cloned_rng.gen::<i32>()%MOD +MOD)%(MOD);
				let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.AVG.to_string().as_str();
				println!("STARTING ELECTION: {:?}", el);
				for addr in locked_user.SERVER_ADDRESSES{
					socket.send_to(&el.as_bytes(), addr).expect("couldn't send data");
				}	
			
			drop(locked_user);
		}else{
			println!("ELECTION NOT STARTED - SERVER INACTIVE");
		}
	});

	let writer_scheduler = timer2.schedule_repeating(chrono::Duration::milliseconds(10000), move || {
		let mut locked_user_no_requests = user1_no_requests.lock().unwrap();
		let mut locked_wtr = wtr1.lock().unwrap();
		locked_wtr.serialize(*locked_user_no_requests);
		locked_wtr.flush();
		drop(locked_wtr);
		println!("req_per_server: {:?}", locked_user_no_requests);
		drop(locked_user_no_requests);
	});

	
	loop {
		
	 	let mut buf = [0; 30];
		let	(amt, src) = socket_.recv_from(&mut buf).expect("Didn't receive data");
	
		if server_up == 0
		{
			continue;
		}
		let mut locked_user = user2.lock().unwrap();

		// check the coming message from a known server. 
	 	if locked_user.SERVER_ADDRESSES.contains(&src.to_string().as_str()) 
		{
			let mut msg = String::from_utf8((&buf).to_vec()).unwrap();


			if amt==1  // If message is 1 byte, then it should be d, u 
			{
				let mut stat = 0;
				if buf[0] =='d' as u8 || buf[0]=='u' as u8
				{
					if buf[0]=='d' as u8 // is the server down?
					{
						if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
							active[0] = 0;
						}
						else{
							active[1] = 0;
						}					
					}
					else if buf[0]=='u' as u8 // is the server up?
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
						locked_user.AVG = (rng.gen::<i32>()%MOD + MOD)%MOD;
						println!("MY AVG: {}",locked_user.AVG);
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
					locked_user.ELECTION[0] = String::new();
					locked_user.ELECTION[1] = String::new();
					locked_user.RESULT.clear();
				}
	 		}
	 		else if msg.split(',').nth(0).unwrap() == "e"{
				//print AVG

				let mut dest = "";
				println!("received e .. MY AVG : {}",locked_user.AVG);
				if active[0] == 0 || active[1] == 0{
					continue;
				}
				msg = msg.split('\0').nth(0).unwrap().to_string();
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
					s1 = locked_user.ELECTION[0].split(',').last().expect("cannot get util from election").parse::<i32>().unwrap();
				}
				if locked_user.ELECTION[1] != ""
				{
					s2 = locked_user.ELECTION[1].split(',').last().expect("cannot get util from election").parse::<i32>().unwrap();
				}
				let b_election = [locked_user.ELECTION[0].as_bytes(),locked_user.ELECTION[1].as_bytes()];	
	
                println!("ELECTION: {:?}", locked_user.ELECTION);

				//if i am in the options already
				if im_considered == 1{
					if locked_user.ELECTION[0]==""{
						//send elec1 
						let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
						socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
					}
					else if locked_user.ELECTION[1]==""{
						let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
						socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
					//not empty and equal
					}else if locked_user.ELECTION[0].split(',').nth(1).unwrap() == locked_user.ELECTION[1].split(',').nth(1).unwrap(){
						println!("----------------------------------");		
						//send out result
						locked_user.RESULT = "r,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + locked_user.ELECTION[0].split(',').nth(2).unwrap();
						println!("SENDING RESULT: {}", locked_user.RESULT);

						locked_user.SERVER_DOWN = locked_user.ELECTION[0].split(',').nth(1).unwrap().to_string();
						if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
							println!("IM GOING DOWN!");
							let down = "d";
							let b_down = down.as_bytes();
							for agent in &agents{

								socket_.send_to(&b_down, agent).expect("couldn't send data");
							}
							for server in &SERVER_ADDRESSES{
								socket_.send_to(&b_down, server).expect("couldn't send data");
							}
							server_up = 0;
							sleep(Duration::from_secs(sleep_s));
							locked_user.AVG = (rng.gen::<i32>()%MOD +MOD)%(MOD);
							println!("MY AVG: {}",locked_user.AVG);
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
					else{ 
						if s1>s2{
							let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
						}else{
							let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
						}
					}

				
				}else
				{	// I am not elected
					if locked_user.ELECTION[0]==""{
						if s2 > locked_user.AVG{
							let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
						}
						else{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.AVG.to_string().as_str();
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}	
						}
					}
					else if locked_user.ELECTION[1]==""{
						if s1 > locked_user.AVG{
							let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");
							
						}
						else{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.AVG.to_string().as_str();
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}
						}
					}else if locked_user.ELECTION[0].split(',').nth(1).unwrap() == locked_user.ELECTION[1].split(',').nth(1).unwrap(){
						if locked_user.AVG > s1{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.AVG.to_string().as_str();
							for addr in locked_user.SERVER_ADDRESSES{
								socket_.send_to(&el.as_bytes(), addr).expect("couldn't send data");
							}
						}else{
							println!("----------------------------------");			
							locked_user.RESULT = "r,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + locked_user.ELECTION[0].split(',').nth(2).unwrap();
							locked_user.SERVER_DOWN = locked_user.ELECTION[0].split(',').nth(1).unwrap().to_string();
							if locked_user.SERVER_DOWN == locked_user.MY_ADDRESS{
								println!("IM GOING DOWN!");
								let down = "d";
								let b_down = down.as_bytes();
								for agent in &agents{

									socket_.send_to(&b_down, agent).expect("couldn't send data");
								}
								for server in &SERVER_ADDRESSES{
									socket_.send_to(&b_down, server).expect("couldn't send data");
								}
								server_up = 0;
								sleep(Duration::from_secs(sleep_s));
								locked_user.AVG = (rng.gen::<i32>()%MOD +MOD)%(MOD);
								println!("MY AVG: {}",locked_user.AVG);
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
					else
					{ 
						if s1 == s2 && locked_user.AVG <= s1{
							//send last
							if src.to_string().as_str() == locked_user.SERVER_ADDRESSES[0]{
								socket_.send_to(&locked_user.ELECTION[0].as_bytes(), dest).expect("couldn't send data");
							}
							else{
								socket_.send_to(&locked_user.ELECTION[1].as_bytes(), dest).expect("couldn't send data");
							}
						}
						else if s1 > s2 && s1 > locked_user.AVG{
							let el = "e,".to_owned() + locked_user.ELECTION[0].split(',').nth(1).unwrap() + "," + s1.to_string().as_str();
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");

						}else if s2 > s1 && s2 > locked_user.AVG{
							let el = "e,".to_owned() + locked_user.ELECTION[1].split(',').nth(1).unwrap() + "," + s2.to_string().as_str();
							socket_.send_to(&el.as_bytes(), dest).expect("couldn't send data");

						}else if locked_user.AVG > s1 && locked_user.AVG > s2{
							let el = "e,".to_owned() + locked_user.MY_ADDRESS + "," + locked_user.AVG.to_string().as_str();
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

		}
}