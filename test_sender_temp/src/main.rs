use std::thread;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use tokio::time::timeout; 
use std::time::{SystemTime, UNIX_EPOCH};
use csv::Writer;
use serde::Serialize;
use std::error::Error;

// Get current time in ms
fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

// Holds statistics for each 
#[derive(Serialize)]
struct thread_data{
    sum_response_time: u32,
    number_of_received_responses: u128,
    number_of_sent_requests: u128,
}

fn main()  -> Result<(), Box<dyn Error>>{
    let temp: u32 = 0;
    let finished_threads = Arc::new(Mutex::new(temp));
    let wtr = Arc::new(Mutex::new(Writer::from_path("client_data.csv")?));
    let wtr1 = wtr.clone();
    for j in 0..500{
        let finished_threads1 = finished_threads.clone();
        let wtr1 = wtr.clone();
        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            let mut local_ctr = 0;
            let mut thread_data_local = thread_data{sum_response_time: 0, number_of_received_responses: 0, number_of_sent_requests: 0};
            loop{
                let start = get_epoch_ms();
                socket.set_read_timeout(Some(std::time::Duration::from_millis(3000))).expect("set_read_timeout call failed");
                socket.send_to(&[1,2,3], "127.0.0.1:4245").expect("couldn't send data");
                thread_data_local.number_of_sent_requests += 1;

                let mut buf = [0;3];
                
                match socket.recv_from(&mut buf){
                    Ok((number_of_bytes, client_address)) => {
                        thread_data_local.number_of_received_responses += 1;
                        let end = get_epoch_ms();
                        let response_time = end-start;
                        thread_data_local.sum_response_time += response_time as u32;
                    },
                    Err(e) => {
                    }
                }
                local_ctr += 1;
            }
            
            let mut locked_wtr = wtr1.lock().unwrap();
            locked_wtr.serialize(thread_data_local);
            drop(locked_wtr);

            let mut locked_user = finished_threads1.lock().unwrap();
            *locked_user += 1;
            println!("finished threads: {}", *locked_user);
            drop(locked_user);
            
            
        });
    }
    let finished_threads1 = finished_threads.clone();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut locked_user = finished_threads1.lock().unwrap();
        if *locked_user == 500
        {
            drop(locked_user);
            break;
        }
        drop(locked_user);
    }

    let mut locked_wtr = wtr1.lock().unwrap();
    locked_wtr.flush();
    drop(locked_wtr);
    Ok(())
}