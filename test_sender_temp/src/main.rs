use std::thread;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use tokio::time::timeout; 
use std::time::{SystemTime, UNIX_EPOCH};
use csv::Writer;
use serde::Serialize;
use std::error::Error;


fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[derive(Serialize)]
struct thread_data{
    sum_response_time: u32,
    number_of_received_responses: u32,
}

fn main()  -> Result<(), Box<dyn Error>>{
    let temp: u32 = 0;
    let finished_threads = Arc::new(Mutex::new(temp));
    let wtr = Arc::new(Mutex::new(Writer::from_path("client_data.csv")?));
    // let mut thread_data_arr: [thread_data;500];
    let wtr1 = wtr.clone();
    for j in 0..500{
        let finished_threads1 = finished_threads.clone();
        let wtr1 = wtr.clone();
        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
            let mut local_ctr = 0;
            let mut thread_data_local = thread_data{sum_response_time: 0, number_of_received_responses: 0};
            for i in 0..10000{
                let start = get_epoch_ms();
                socket.set_read_timeout(Some(std::time::Duration::from_millis(3000))).expect("set_read_timeout call failed");
                socket.send_to(&[1,2,3], "127.0.0.1:4245").expect("couldn't send data");
                // println!("port: {}",socket.local_addr().unwrap().port());
                let mut buf = [0;3];
                // receive timeout in socket
                // try and catch
                match socket.recv_from(&mut buf){
                    Ok((number_of_bytes, client_address)) => {
                        //println!("received: {:?}", buf);
                        thread_data_local.number_of_received_responses += 1;
                        let end = get_epoch_ms();
                        let response_time = end-start;
                        thread_data_local.sum_response_time += response_time as u32;
                    },
                    Err(e) => {
                        //println!("error: {:?}", e);
                    }
                }
                //socket.recv_from(&mut buf).expect("Didn't receive data");
                local_ctr += 1;
            }
            
            let mut locked_wtr = wtr1.lock().unwrap();
            locked_wtr.serialize(thread_data_local);
            drop(locked_wtr);

            let mut locked_user = finished_threads1.lock().unwrap();
            *locked_user += 1;
            println!("finished threads: {}", *locked_user);
            drop(locked_user);
            
            
            // println!("Arr: {:?}", buf);
        });
    }
    let finished_threads1 = finished_threads.clone();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // println!("Received: {}", received);
        let mut locked_user = finished_threads1.lock().unwrap();
        if *locked_user == 500
        {
            drop(locked_user);
            break;
        }
        drop(locked_user);
    }
    
    // wtr.expect("writer failed").serialize(&sum_response_time);
    // wtr.expect("writer failed").serialize(&number_of_received_responses);
    // let wtr1 = wtr.clone();
    let mut locked_wtr = wtr1.lock().unwrap();
    locked_wtr.flush();
    drop(locked_wtr);
    Ok(())
}