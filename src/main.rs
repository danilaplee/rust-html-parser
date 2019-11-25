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
use std::any::Any;
use json::object;

//REDIS KEYS
static tgnews_nlu_reply:&'static str = "tgnews_nlu_reply_list";
static tgnews_nlu_request:&'static str = "tgnews_nlu_request_list";
static tgnews_nlu:&'static str = "tgnews_nlu";
static tgnews_nlu_start:&'static str = "tgnews_nlu_start";
static tgnews_nlu_end:&'static str = "tgnews_nlu_end";
static tgnews_nlu_reply_timeout:usize = 5;

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
    // println!("redis data {:?}", data);
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

	let mut pubsub = con.as_pubsub();
	pubsub.subscribe(tgnews_nlu_start);
	if query == "debug" {
		let python_process = Command::new("python3")
		.arg("/Users/danilapuzikov/dev/clones/telegram/python/nlu_service.py")
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
		.spawn()
		.expect("failed to execute process");
		println!("subscribed to tgnews_nlu");
	}
	else {
		let python_process = Command::new("python3")
		.arg("/Users/danilapuzikov/dev/clones/telegram/python/nlu_service.py")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
		.spawn()
		.expect("failed to execute process");
	}

	loop {
	    let msg = pubsub.get_message()?;
	    let payload : String = msg.get_payload()?;
	    if payload == "listener_started" {
	    	pubsub.unsubscribe(tgnews_nlu_start)?;
	    	return Ok(());
	    }
	}
}

async fn wait_for_nlu_completion(con: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error + 'static>>   {

	let mut pubsub = con.as_pubsub();
	pubsub.subscribe(tgnews_nlu_end);
	loop {
	    let msg = pubsub.get_message()?;
	    let payload : String = msg.get_payload()?;
	    if payload == "done" {
	    	pubsub.unsubscribe(tgnews_nlu_end)?;
	    	return Ok(());
	    }
	}
}

fn run_nlu_listener() -> Result<(), Box<dyn std::error::Error + 'static>> {
    thread::spawn( || {
	    let args: Vec<String> = env::args().collect();
	    let query	 = &args[1];
	    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
	    let mut con = client.get_connection().unwrap();
	    loop {
			let res:Result<(String, String), redis::RedisError> = con.brpop(tgnews_nlu_reply, tgnews_nlu_reply_timeout);
			match res {
			    Ok(t) => { 
			    	let (key, value) = t;
			    	println!("nlu response data {}", value);

			    	let data = json::parse(&value);
			    	match data {
			    		Ok(json_data) => {
					    	println!("nlu response json {:?}", json_data);
			    		},
			    		Err(e) => {
					        if query == "debug" {
						        println!("nlu response json error: {}", e);
						    }
			    		}
			    	}
			    },
			    Err(e) => { 
			        if query == "debug" {
			        	println!("nlu response error: {:?}", e.to_string());
			        }
			        if e.to_string() == r#"Response was of incompatible type: "Not a bulk response" (response was nil)"# {
				        if query == "debug" {
				        	println!("end of line");
				        }
					    let _ : () = con.publish(tgnews_nlu_end, "done".to_string()).unwrap();
					    return;
			        }
			    },
			}
	    }

    });
	Ok(())
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let query	 = &args[1];
    let filename = &args[2];
	
    //CLEAN DB SYNC
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let p1: () = con.publish(tgnews_nlu, "done").unwrap();
    delete_set(&mut con, "eng".to_string());
    delete_set(&mut con, "rus".to_string());
    delete_set(&mut con, tgnews_nlu_reply.to_string());
    delete_set(&mut con, tgnews_nlu_request.to_string());

	let start_time 	= Utc::now();
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
	    run_nlu_listener();
	}
    

	let start 		= Instant::now();

    //START DIRS
    let path = Path::new(filename);
    let result = visit_dirs(path);
	

	//GET RESULT DATA
	if query == "languages" {
		print_languages_end(&mut con);
		return;
	}

	if query == "debug" {
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		block_on(wait_for_nlu_completion(&mut con));
		//COUNT PERFORMANCE
		let end_time = Utc::now();
		let duration = start.elapsed();
	    println!("=============== ALL DONE! ===============");
	    println!("=============== END TIME {} ===============", end_time);
	    println!("=============== DURATION {:?} ===============", duration);
	}
	//SETUP NEWS
	if query == "news" {
		block_on(wait_for_nlu_completion(&mut con));
	}
    let p2:() = con.publish(tgnews_nlu, "done").unwrap();

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
			    let args: Vec<String> = env::args().collect();
			    let query = &args[1];
			    
			    let mut _millis = time::Duration::from_millis(10);

				if query == "languages" {
				    _millis = time::Duration::from_millis(1);
				}
				if query == "news" {
				    _millis = time::Duration::from_millis(5);
				}
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
	    let lang_data = object!{
	    	"h1" => h1,
	    	"path" => pstr
	    };
	    con.publish(tgnews_nlu, lang_data.dump())?;
    }
    Ok(())

}

