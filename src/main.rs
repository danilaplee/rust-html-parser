extern crate whatlang;
extern crate select;
extern crate futures;
extern crate chrono;
extern crate redis;

use chrono::{Datelike, Timelike, Utc};
use std::time::{Duration, Instant};
use std::io;
use std::fs::{self, DirEntry};
use std::fs::File;
use std::path::Path;
use std::env;
use std::thread;
use std::io::BufReader;
use std::net::SocketAddr;
use whatlang::{detect, Lang, Script};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use futures::executor::block_on;
use redis::Commands;
use std::path::PathBuf;
use std::time;

fn deleteSet(con: &mut redis::Connection, ntype: String) -> redis::RedisResult<()> {
    let _ : () = redis::cmd("DEL").arg(ntype).query(con)?;
    Ok(())
}
fn addToSet(con: &mut redis::Connection, ntype: &String, nitem: &String) -> redis::RedisResult<()> {
    let _ : () = redis::cmd("SADD").arg(ntype).arg(nitem).query(con)?;
    Ok(())
}

fn main() {
	let start_time = Utc::now();
	let start = Instant::now();
    println!("=============== RUNNING TGNEWS v0.2.0 ===============");
    println!("=============== START TIME {} ===============", start_time);
    let args: Vec<String> = env::args().collect();

    let query = &args[1];
    let filename = &args[2];

    println!("Searching for {}", query);
    println!("In folder {}", filename);
    
    //CLEAN DB SYNC
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    deleteSet(&mut con, "eng".to_string());
    deleteSet(&mut con, "rus".to_string());

    //START DIRS
    let path = Path::new(filename);
    let result = visit_dirs(path);
	
	//START PERFORMANCE
	let end_time = Utc::now();
	let duration = start.elapsed();


	if query == "debug" {
	    println!("=============== ALL DONE! ===============");
	    println!("=============== END TIME {} ===============", end_time);
	    println!("=============== DURATION {:?} ===============", duration);
	}

}

fn visit_dirs(dir: &Path) -> io::Result<()> {

    let mut ittr = 0;
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
	    	ittr += 1;
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
            	visit_dirs(&path);
            } else {
			    thread::spawn(move || {
				    let args: Vec<String> = env::args().collect();
				    let query = &args[1];
			    	
			    	if query == "debug" {
			    		println!("spawned a new thread {} for dir", ittr);
			    	}

		            parse_file(&entry);
			    });
			    let ten_millis = time::Duration::from_millis(1);
				let now = time::Instant::now();

				thread::sleep(ten_millis);
            }
        }
    }
    Ok(())
}


fn parse_file(entry: &DirEntry) -> Result<(), Box<dyn std::error::Error + 'static>> {

    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    let path = entry.path();
    let pstr:String = String::from(path.as_path().to_str().unwrap());
	if query == "debug" {
		println!("parsing File {:?}", path);
	}
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let document = Document::from_read(reader).unwrap();
    let mut h1 : String = "1".to_string();

    for node in document.find( Name("h1") ) {
        h1 = node.text();
    }
    let info = detect(&h1).unwrap();
    let eng = info.lang() == Lang::Eng;
    let rus = info.lang() == Lang::Rus;

    if eng || rus {
	    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
	    let mut con = client.get_connection().unwrap();
	    let mut key = "eng";
	    if rus {
	    	key = "rus"
	    }
	    addToSet(&mut con, &key.to_string(), &pstr);
		if query == "debug" {
	    	println!("{}", &key);
		}
    }
    Ok(())

}

