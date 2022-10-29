use std::thread;
use std::time::Duration;
use nix::{sys::wait::waitpid,unistd::{fork, ForkResult, write}};


fn main() {
use std::net::UdpSocket;

// create an array of strings
let listener_addresses = ["127.0.0.1:4242", "127.0.0.1:4243", "127.0.0.1:4244"];

let socket = UdpSocket::bind("127.0.0.1:12346").expect("couldn't bind to address");

let random_arrays = [[1,2,3], [4,5,6], [7,8,9]];
let mut threads = Vec::new();

// Creating a child fork
match unsafe{fork()} {
    Ok(ForkResult::Parent { child, .. }) => {
        // Parent process
        println!("Parent process: {}", child);
        // Wait for child process to finish
        let child_pid = waitpid(child, None).unwrap();
        println!("Child process {:?} finished", child_pid);
    }
    Ok(ForkResult::Child) => {
        // Child process
        println!("Child process");
        // Create a thread for each listener
        println!("Creating threads");
        for i in 0..30 {
            let socket = socket.try_clone().expect("couldn't clone socket");
            let listener_address = listener_addresses[i%3].to_string();
            let random_array = random_arrays[i%3];
            let temp = thread::spawn(move || {
                socket.send_to(&random_array, listener_address).expect("couldn't send data");
                let mut buf = [0;3];
                let (number_of_bytes, _) = socket.recv_from(&mut buf)
                                                .expect("Didn't receive data");
                let filled_buf = &mut buf[..number_of_bytes];
                println!("Arr: {:?}",filled_buf);
            });
            threads.push(temp);
        }
        // Wait for all threads to finish
        for thread in threads {
            thread.join().unwrap();
        }
    }
    Err(_) => {
        // Error
        println!("Error");
    }
}




}

