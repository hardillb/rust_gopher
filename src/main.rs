// use std::fs;
use std::env;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

struct Gophermap {
	row_type: char,
	label: String,
	path: String,
	server: String,
	port: i16
}

fn main() -> std::io::Result<()> {

    println!("Hello, world!");
	let listener = TcpListener::bind("0.0.0.0:70").unwrap();
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_connection(stream?).unwrap();
    }
    Ok(())

}

fn read_gophermap(path: &Path) -> Vec<Gophermap>{
	let mut entries: Vec<Gophermap> = Vec::new();

	let file = File::open(path).unwrap();
	let reader = BufReader::new(file);

	for line in reader.lines() {
		let mut l = line.unwrap();
		let mut t = 'i';
		let mut label = l.to_string();
		let mut p = "fake";
		let mut s = "(NULL)";

		if l.starts_with("0") {
 			t = '0';
 			l.remove(0);
 			let parts: Vec<&str> = l.split('\t').collect();
 			println!("{:?}",parts);
 			label = parts[0].to_string();
 			p = parts[1];
 			s = "localhost";

 		} else if l.starts_with("1") {
 			t = '1';
 			l.remove(0);
 			let parts: Vec<&str> = l.split('\t').collect();
 			println!("{:?}",parts);
 			label = parts[0].to_string();
 			p = parts[1];
 			s = "localhost";
 		} 
 		// println!("line - {}", l);
 		let  entry = Gophermap {
 			row_type: t,
 			label: label,
 			path: p.to_string(),
 			server: s.to_string(),
 			port: 70
 		};

 		entries.push(entry);
	}

	entries
}

fn handle_connection(stream: TcpStream) -> std::io::Result<()> {
	let mut request = String::new(); 

	println!("Connection from {}", stream.peer_addr().unwrap());

	let mut reader = BufReader::new(stream.try_clone().unwrap());
	reader.read_line(&mut request).unwrap();
	let request = request.trim_end();


	if request.ends_with("$") {
		handle_plus(stream, request).unwrap();
	} else {
		handle_v1(stream, request).unwrap();
	}

	Ok(())
}

fn handle_v1(mut stream: TcpStream, request: &str) -> std::io::Result<()> {
	let mut dir = env::current_dir().unwrap();
	// println!("CWD {}", dir.to_str().unwrap());

	dir.push("root");
	// println!("root dir {}", dir.to_str().unwrap());
	let request_path = Path::new(request);
	if request.starts_with("/") {
		let request_path = request_path.strip_prefix("/").unwrap();
		dir.push(request_path);
	}
	let path = dir.as_path();
	// println!("requested path {}", path.to_str().unwrap());

	if path.exists() {
		if path.is_dir() {
			let map = path.join("gophermap");
			// println!("map path {}",map.to_str().unwrap());
			if map.exists() {
				let lines = read_gophermap(&map);
				for l in lines {
					stream.write(format!("{}{}\t{}\t{}\t{}\r\n", l.row_type, l.label, l.path, l.server, l.port).as_bytes()).expect("failed to send");
				}
			}
		} else {
			stream.write(&request.as_bytes()).expect("Failed to send");
		} 
	} else {
		let error = format!("3'{}' not found\r\n", request);
		stream.write(&error.as_bytes()).expect("failed to send");
	}

	stream.flush().unwrap();
	Ok(())
}

fn handle_plus(mut stream: TcpStream, request: &str) -> std::io::Result<()> {
	let mut dir = env::current_dir().unwrap();
	dir.push("root");

	let parts: Vec<&str> = request.split('\t').collect();
	// println!("root dir {}", dir.to_str().unwrap());
	let request_path = Path::new(parts[0]);
	if parts[0].starts_with("/") {
		let request_path = request_path.strip_prefix("/").unwrap();
		dir.push(request_path);
	}
	let path = dir.as_path();

	println!("requested path {}", path.to_str().unwrap());

	if path.exists() {
		if path.is_dir() {
			let map = path.join("gophermap");
			if map.exists() {
				let lines = read_gophermap(&map);
				stream.write("+-2".as_bytes()).expect("failed to send");
				for l in lines {
					stream.write(format!("+INFO: {}{}\t{}\t{}\t{}\r\n", l.row_type, l.label, l.path, l.server, l.port).as_bytes()).expect("failed to send");
				}
			}
		} else {

		}
	}

	//let data = "+-2\r\n+INFO: iBen's Place - Gopher\tfake\t(NULL)\t0\r\n+INFO: i\tfake\t(NULL)\t0\r\n+INFO: 0CV /cv.txt\tlocalhost\t70\t+\r\n";
	//stream.write(&data.as_bytes()).expect("Failed to send");

	stream.flush().unwrap();
	Ok(())
}