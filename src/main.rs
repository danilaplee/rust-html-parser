extern crate whatlang;
extern crate select;
extern crate futures;
extern crate chrono;
extern crate redis;

use chrono::{Datelike, Timelike, Utc};
use std::time::{Duration, Instant};
use std::io;
use std::str;
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
use std::collections::HashSet;
use std::path::PathBuf;
use std::time;
use std::process::{Command, Stdio};

fn delete_set(con: &mut redis::Connection, ntype: String) -> redis::RedisResult<()> {
    let _ : () = redis::cmd("DEL").arg(ntype).query(con)?;
    Ok(())
}

fn add_to_set(con: &mut redis::Connection, ntype: &String, nitem: &String) -> redis::RedisResult<()> {
    let _ : () = redis::cmd("SADD").arg(ntype).arg(nitem).query(con)?;
    Ok(())
}

fn get_set(con: &mut redis::Connection, ntype: &String) -> redis::RedisResult<()> {
    let data = redis::cmd("SMEMBERS").arg(ntype).query(con)?;
    println!("redis data {:?}", data);
    Ok(())
}
fn print_languages_start() {

		let lang_en = r#"[
	{
		"lang_code": "en",
		"articles": ["#;

		println!("{}", lang_en);
}
fn print_languages_end(con: &mut redis::Connection) {

		let rus_data : HashSet<String> = con.smembers("rus".to_string()).unwrap();

		let lang_ru = r#"		""
		]
	},
	{
		"lang_code": "ru",
		"articles": ["#;
		let lang_end = r#"		]
	}
]
		"#;
		println!("{}", lang_ru);
		for item in rus_data {
			println!(r#"		{:?}, "#, item);
		}
		println!(r#"		"""#);
		println!("{}", lang_end);
}

async fn run_python_service(con: &mut redis::Connection, query: &str) -> Result<(), Box<dyn std::error::Error + 'static>>   {


	let output = Command::new("python3")
	            .arg("/Users/danilapuzikov/dev/clones/telegram/python/nlu_service.py")
		        .stdout(Stdio::null())
		        .stderr(Stdio::null())
	            .spawn()
	            .expect("failed to execute process");

	let mut pubsub = con.as_pubsub();
	pubsub.subscribe("tgnews_nlu_reply");

	println!("subscribed to tgnews_nlu");

	loop {
	    let msg = pubsub.get_message()?;
	    let payload : String = msg.get_payload()?;
	    if payload == "listener_started" {
	    	pubsub.unsubscribe("tgnews_nlu")?;
	    	return Ok(());
	    }
	}
}

fn run_nlu_listener() -> Result<(), Box<dyn std::error::Error + 'static>> {
    thread::spawn( || {
	    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
	    let mut con = client.get_connection().unwrap();
		let mut pubsub = con.as_pubsub();
		pubsub.subscribe("tgnews_nlu_reply");
		loop {
		    let msg = pubsub.get_message().unwrap();
		    let payload : String = msg.get_payload().unwrap();
		    println!("{}", payload);
		    if payload == "stop" {
		    	pubsub.unsubscribe("tgnews_nlu_reply");
		    	// Ok(());
		    }
		}

    });
	Ok(())
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let query	 = &args[1];
    let filename = &args[2];

	let start_time 	= Utc::now();
	let start 		= Instant::now();
	
    //CLEAN DB SYNC
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    delete_set(&mut con, "eng".to_string());
    delete_set(&mut con, "rus".to_string());

    //SETUP DEBUG
	if query == "debug" {
	    println!("=============== RUNNING TGNEWS v0.4.2 ===============");
	    println!("=============== START TIME {} ===============", start_time);
	    println!("Searching for {}", query);
	    println!("In folder {}", filename);
	    block_on(run_python_service(&mut con, &query));
	    println!("python setup done");
	    run_nlu_listener();
	    println!("listener setup done");
	}

	//SETUP LANGUAGES
	if query == "languages" {
		print_languages_start();
	}

	//SETUP NEWS
	if query == "news" {
	    block_on(run_python_service(&mut con, &query));
	}
    

	//START PERFORMANCE
	let end_time = Utc::now();

    //START DIRS
    let path = Path::new(filename);
    let result = visit_dirs(path);
	
	//COUNT PERFORMANCE
	let duration = start.elapsed();

	//GET RESULT DATA
	if query == "languages" {
		print_languages_end(&mut con);
	}

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
			    		// println!("spawned a new thread {} for dir", ittr);
			    	}

		            match parse_file(&entry) {
			            Result::Ok(val) => val,
			            Result::Err(err) => return,
		            }
			    });
			    let _millis = time::Duration::from_millis(5);
				let now = time::Instant::now();

				thread::sleep(_millis);
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
	    add_to_set(&mut con, &key.to_string(), &pstr)?;
	    con.set(&pstr, &h1)?;
	    con.publish("tgnews_nlu".to_string(), &h1)?;
    }
    Ok(())

}

