extern crate whatlang;
extern crate select;
extern crate futures;
extern crate chrono;
extern crate redis;
extern crate json;

use chrono::{Utc};
use std::time::{Instant};
use std::io;
use std::str;
use std::fs::{self, DirEntry};
use std::fs::File;
use std::path::Path;
use std::env;
use std::thread;
use std::io::BufReader;
use whatlang::{detect, Lang};
use select::document::Document;
use select::predicate::{Name};
use futures::executor::block_on;
use redis::Commands;
use redis::RedisResult;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time;
use std::process::{Command, Stdio};
use json::object;
use json::JsonValue;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::collections::VecDeque;

use super::tgnews_nlu_reply;
use super::tgnews_nlu_request;
use super::tgnews_nlu;
use super::tgnews_nlu_start;
use super::tgnews_nlu_end;
use super::tgnews_nlu_reply_timeout;
use super::add_to_set;

pub fn visit_dirs(
	dir: &Path, 
	queue:Arc<Mutex<VecDeque<JsonValue>>>,
	ruDB:Arc<Mutex<Vec<String>>>) -> io::Result<()> {


    let mut ittr = 0;
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
	    	ittr += 1;
            let entry = entry?;

            let path = entry.path();
		    let q = Arc::clone(&queue);
		    let rus = Arc::clone(&ruDB);

            if path.is_dir() {
            	visit_dirs(&path, q, rus);
            } else {
			    thread::spawn(move || {
				    let args: Vec<String> = env::args().collect();
				    let query = &args[1];
			    	
			    	if query == "debug" {
			    		// println!("spawned a new thread {} for dir", ittr);
			    	}

		            match parse_file(&entry, q, rus) {
			            Result::Ok(val) => val,
			            Result::Err(err) => return,
		            }
			    });
			    let args: Vec<String> = env::args().collect();
			    let query = &args[1];
			    
			    let mut _millis = time::Duration::from_millis(10);

				if query == "languages" {
				    _millis = time::Duration::from_millis(1);
				}
				if query == "news" {
				    _millis = time::Duration::from_millis(5);
				}

				thread::sleep(_millis);
            }
        }
    }
    Ok(())
}


pub fn parse_file(entry: &DirEntry, 
	queue:Arc<Mutex<VecDeque<JsonValue>>>,
	ruDB:Arc<Mutex<Vec<String>>>) -> Result<(), Box<dyn std::error::Error + 'static>> {

    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    let path = entry.path();
    let pstr:String = String::from(path.as_path().to_str().unwrap());
	
	if query == "debug" {
		// println!("parsing File {:?}", path);
	}

    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let document = Document::from_read(reader)?;

    let mut h1 : String = "1".to_string();

    for node in document.find( Name("h1") ) {
        h1 = node.text();
    }

    let de = detect(&h1);

    if de == None {
    	return Ok(())
    }

    let info = de.unwrap();

    let eng = info.lang() == Lang::Eng;
    let rus = info.lang() == Lang::Rus;

    if eng || rus {
	    let client = redis::Client::open("redis://127.0.0.1/")?;
	    let mut con = client.get_connection()?;
	    let mut key = "rus";
	    if eng {
	    	key = "eng";

			if query == "debug" {
		    	// println!("{}", &h1);
		    	// println!("language: {}", &key);
		    	// println!("path: {}", &pstr);
			}
			if query == "languages" {
				println!(r#"		{:?},"#, &pstr);
			}
	    }
	    let lang_data = object!{
	    	"h1" => h1,
	    	"path" => json::JsonValue::String(pstr.to_string()),
	    	"lang" => key
	    };

	    add_to_set(&mut con, &key.to_string(), &pstr)?;
	    con.publish(tgnews_nlu, &lang_data.dump())?;
	    let mut lock = queue.try_lock();
	    if let Ok(ref mut mtx) = lock {
	        // println!("total queue length: {:?}", mtx.len());
	       	mtx.push_back(lang_data);
	    } else {
	        // println!("parser first try_lock failed");
	    }
	    drop(lock);
	    if key == "rus" {
		    let mut lock2 = ruDB.try_lock();
		    if let Ok(ref mut mtx2) = lock2 {
		        // println!("total queue length: {:?}", mtx.len());
		       	mtx2.push(pstr);
		    } else {
		        // println!("parser second try_lock failed");
		    }
		    drop(lock2);
	    }
	    // println!("total size of queue: {:?}", queue.add_work(&lang_data));
    }
    Ok(())

}
