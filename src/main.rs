// use std::fs;
use std::io;
use std::env;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use clap::{Arg, App};

struct Gophermap {
	row_type: char,
	label: String,
	path: String,
	server: String,
	port: i16
}

struct Config {
	hostname: String,
	port: i16,
	dir: String
}

fn main() -> std::io::Result<()> {

	let matches = App::new("rust_gopher")
		.version("0.1.0")
		.author("Ben Hardill")
		.about("Basic Gopher Server")
		.arg(Arg::with_name("hostname")
			.short("h")
			.long("hostname")
			.takes_value(true)
			.help("The hostname of this server"))
		.arg(Arg::with_name("port")
			.short("p")
			.long("port")
			.takes_value(true)
			.help("Port number to listen on"))
		.arg(Arg::with_name("dir")
			.short("d")
			.long("dir")
			.takes_value(true)
			.help("path to gopher content"))
		.get_matches();

	let hostname = matches.value_of("hostname").unwrap_or("localhost");
	let port :i16 = matches.value_of("port").unwrap_or("70").parse().unwrap();
	let dir = matches.value_of("dir").unwrap_or("root");
	println!("Listening on 0.0.0.0 and port {} as {}", port, hostname);

	let bind_addr = format!("0.0.0.0:{}", port);

	let config = Config {
		hostname: hostname.to_string(),
		port: port,
		dir: dir.to_string()
	};

	let listener = TcpListener::bind(bind_addr).unwrap();
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_connection(stream?, &config).unwrap();
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, config: &Config) -> std::io::Result<()> {
	let mut request = String::new(); 

	println!("Connection from {}", stream.peer_addr().unwrap());

	let mut reader = BufReader::new(stream.try_clone().unwrap());
	reader.read_line(&mut request).unwrap();
	let request = request.trim_end();


	if request.ends_with("$") {
		handle_plus(stream, request, config).unwrap();
	} else {
		handle_v1(stream, request, config).unwrap();
	}
	Ok(())
}


fn read_gophermap(path: &Path, config: &Config) -> Vec<Gophermap>{
	let mut entries: Vec<Gophermap> = Vec::new();

	let file = File::open(path).unwrap();
	let reader = BufReader::new(file);

	for line in reader.lines() {
		let mut l = line.unwrap();
		let mut t = 'i';
		let mut label = l.to_string();
		let mut p = "fake";
		let mut s = "(NULL)";
		let mut port = config.port;

		if l.starts_with("0") {
 			t = '0';
 			l.remove(0);
 			let parts: Vec<&str> = l.split('\t').collect();
 			// println!("{:?}",parts);
 			label = parts[0].to_string();
 			p = parts[1];

 			s = &config.hostname;

 		} else if l.starts_with("1") {
 			t = '1';
 			l.remove(0);
 			let parts: Vec<&str> = l.split('\t').collect();
 			// println!("{:?}",parts);
 			label = parts[0].to_string();
 			p = parts[1];
 			s = &config.hostname;
 		} 
 		// println!("line - {}", l);
 		let  entry = Gophermap {
 			row_type: t,
 			label: label,
 			path: p.to_string(),
 			server: s.to_string(),
 			port: port
 		};

 		entries.push(entry);
	}

	entries
}

fn handle_v1(mut stream: TcpStream, request: &str, config: &Config) -> std::io::Result<()> {
	let mut dir = env::current_dir().unwrap();

	dir.push(&config.dir);
	// println!("{}", dir.to_str().unwrap());
	let mut request_path = Path::new(request);
	if request.starts_with("/") {
		request_path = request_path.strip_prefix("/").unwrap();
	}
	dir.push(request_path);
	let path = dir.as_path();

	if path.exists() {
		if path.is_dir() {
			let map = path.join("gophermap");
			// println!("map path {}",map.to_str().unwrap());
			if map.exists() {
				let lines = read_gophermap(&map, config);
				for l in lines {
					stream.write(format!("{}{}\t{}\t{}\t{}\r\n", l.row_type, l.label, l.path, l.server, l.port).as_bytes()).expect("failed to send");
				}
			}
		} else {
			let mut file = File::open(path).unwrap();
			io::copy(&mut file, &mut stream).unwrap();
		} 
	} else {
		let error = format!("3'{}' not found\r\n", request);
		stream.write(&error.as_bytes()).expect("failed to send");
	}

	stream.flush().unwrap();
	Ok(())
}

fn handle_plus(mut stream: TcpStream, request: &str, config: &Config) -> std::io::Result<()> {
	let mut dir = env::current_dir().unwrap();
	dir.push(&config.dir);

	let parts: Vec<&str> = request.split('\t').collect();
	// println!("root dir {}", dir.to_str().unwrap());
	let request_path = Path::new(parts[0]);
	if parts[0].starts_with("/") {
		let request_path = request_path.strip_prefix("/").unwrap();
		dir.push(request_path);
	}
	let path = dir.as_path();

	if path.exists() {
		if path.is_dir() {
			let map = path.join("gophermap");
			if map.exists() {
				let lines = read_gophermap(&map, config);
				stream.write("+-2".as_bytes()).expect("failed to send");
				for l in lines {
					stream.write(format!("+INFO: {}{}\t{}\t{}\t{}\r\n", l.row_type, l.label, l.path, l.server, l.port).as_bytes()).expect("failed to send");
				}
			}
		} else {

		}
	}

	stream.flush().unwrap();
	Ok(())
}