// use std::time::Instant;
// use std::time::Duration;
// use std::time::SystemTime;
// use sysinfo::{CpuExt, System, SystemExt};
// use sys_metrics::{cpu::*};
// use std::time::Timer;
use timer::Timer;

// extern crate cpu;
// use cpu::Cpu;
fn startelection(){
	println!("start election fn");

	let mut sys = System::new();
    sys.refresh_cpu(); // Refreshing CPU information.
	for cpu in sys.cpus() {
		print!("{}% ", cpu.cpu_usage());
	}
}
use sysinfo::{CpuExt, System, SystemExt};

fn main() {
    use std::net::UdpSocket;

	let socket = UdpSocket::bind("127.0.0.1:4242").expect("couldn't bind to address");
	// let serverAddresses = vec!["127.0.0.1:4243","127.0.0.1:4244"];
    // let mut buf = [0;3];
    let mut agents: Vec<String> = Vec::new();

	let timer = Timer::new();

    let _timer_handle = timer.schedule_repeating(chrono::Duration::milliseconds(5000), || {
		println!("Timer fired!");
		startelection();
	});


	loop {
    	let mut buf = [0; 3];
    	let (amt, src) = socket.recv_from(&mut buf).expect("Didn't receive data");
    	println!("{} bytes from {}", amt, src);
    	println!("data: {:?}", buf);

    	if !agents.contains(&src.to_string()) {
    		agents.push(src.to_string());
		}

    	println!("agents: {:?}", agents);
    }
}