use std::thread;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use tokio::time::timeout; 
use std::time::{SystemTime, UNIX_EPOCH};
use csv::Writer;
use serde::Serialize;
use std::error::Error;
use timer::Timer;

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
    sum_response_time: u128,
    number_of_received_responses: u128,
    number_of_sent_requests: u128,
}

fn main()  -> Result<(), Box<dyn Error>>{
    for j in 0..500{
        
        thread::spawn(move || -> Result<(), std::io::Error>{
            let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            let wtr = Arc::new(Mutex::new(Writer::from_path("client_data".to_owned() + j.to_string().as_str() + ".csv")?));
            let thread_data_local = Arc::new(Mutex::new(thread_data{sum_response_time: 0, number_of_received_responses: 0, number_of_sent_requests: 0}));
            let wtr1 = wtr.clone();
            let thread_data_local1 = thread_data_local.clone();
            let timer = Timer::new();
            let handle = timer.schedule_repeating(chrono::Duration::milliseconds(10000), move || {
                let mut locked_wtr = wtr1.lock().unwrap();
                let mut locked_thread_data_local = thread_data_local1.lock().unwrap();
                let mut thread_data_temp = thread_data{
                    sum_response_time: locked_thread_data_local.sum_response_time,
                    number_of_received_responses: locked_thread_data_local.number_of_received_responses,
                    number_of_sent_requests: locked_thread_data_local.number_of_sent_requests,
                };
                locked_wtr.serialize(thread_data_temp);
                drop(locked_thread_data_local);
                locked_wtr.flush();
                drop(locked_wtr);
            });
            let thread_data_local2 = thread_data_local.clone();
            loop{
                
                
                let start = get_epoch_ms();
                socket.set_read_timeout(Some(std::time::Duration::from_millis(3000))).expect("set_read_timeout call failed");
                socket.send_to(&[1,2,3], "127.0.0.1:4245").expect("couldn't send data");
                let mut locked_thread_data_local = thread_data_local2.lock().unwrap();
                locked_thread_data_local.number_of_sent_requests += 1;
                drop(locked_thread_data_local);

                let mut buf = [0;3];
                
                match socket.recv_from(&mut buf){
                    Ok((number_of_bytes, client_address)) => {
                        let end = get_epoch_ms();
                        let response_time = end-start;
                        let mut locked_thread_data_local = thread_data_local2.lock().unwrap();
                        locked_thread_data_local.sum_response_time += response_time as u128;
                        locked_thread_data_local.number_of_received_responses += 1;
                        drop(locked_thread_data_local);
                    },
                    Err(e) => {
                    }
                }
                
            }
            
        });
        
    }
    loop{}
    Ok(())
}
