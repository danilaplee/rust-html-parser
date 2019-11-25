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
use json::JsonValue;

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
    Ok(())
}
fn print_languages_start() {

		let lang_en = r#"[
	{
		"lang_code": "en",
		"articles": ["#;

		println!("{}", lang_en);
}

fn print_news_start() {

		let start = r#"{
		"articles": ["#;

		println!("{}", start);
}

fn print_news_end() {

		let end = r#"		]
}"#;

		println!("{}", end);
}
fn print_languages_end(con: &mut redis::Connection) {

		let rus_data : HashSet<String> = con.smembers("rus".to_string()).unwrap();

		let lang_ru = r#"		]
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
		println!("{}", lang_end);
}

async fn run_nlu_service(con: &mut redis::Connection, query: &str) -> Result<(), Box<dyn std::error::Error + 'static>>   {

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
fn process_nlu_data(data:JsonValue) {
	
    let args:Vec<String>= env::args().collect();
    let query	 		= &args[1];
	let words 			= &data["response"][0][0];
	let meaning 		= &data["response"][1][0];
	let gpe_key 		= "GPE".to_string();
	let org_key 		= "ORG".to_string();
	let person_key 		= "PERSON".to_string();
	let work_of_art_key = "WORK_OF_ART".to_string();
	let money_key 		= "MONEY".to_string();
	let product_key 	= "PRODUCT".to_string();
	let loc_key 		= "LOC".to_string();

	let mut current 	= 0;
	let mut gpe 		= false;
	let mut org 		= false;
	let mut work_of_art = false;
	let mut person 		= false;
	let mut ordinal 	= false;
	let mut event 		= false;
	let mut location 	= false;
	let mut money 		= false;
	let mut product 	= false;
	let mut is_news 	= false;

	while current <= meaning.len() {
		let m = &meaning[current];
		let mss = m.to_string();
		let ms: Vec<&str> = mss.split("-").collect();
		if ms.len() > 1 {
			let s = ms[1].to_string();
			if s == gpe_key {
				gpe = true;
			}
			if s == org_key {
				org = true;
			}
			if s == work_of_art_key  {
				work_of_art = true;
			}
			if s == person_key  {
				person = true;
			}
			if s == product_key  {
				product = true;
			}
			if s == money_key  {
				money = true;
			}
			if s == loc_key  {
				location = true;
			}
		}
		current += 1;
	}

	if !is_news && (org || gpe || person) && money && is_news {
		is_news = true;
	}
	if !is_news && org && gpe {
		is_news = true;
	}
	if !is_news && work_of_art {
		is_news = true;
	}
	if is_news {
		if query == "debug" {
			println!("news worthy: {:?}", data["h1"].to_string());
		}
		if query == "news" {
			println!(r#"			{:?},"#, &data["path"].to_string());
		}
	}
	let mut ndata 	= json::JsonValue::new_object();
	ndata["news"] 	= json::JsonValue::Boolean(is_news);
	ndata["org"] 	= json::JsonValue::Boolean(org);
	ndata["gpe"] 	= json::JsonValue::Boolean(gpe);
	ndata["person"] = json::JsonValue::Boolean(person);
	ndata["product"]= json::JsonValue::Boolean(product);
	ndata["money"] 	= json::JsonValue::Boolean(money);
	ndata["loc"]	= json::JsonValue::Boolean(location);
	ndata["art"] 	= json::JsonValue::Boolean(work_of_art);
	ndata["path"] 	= json::JsonValue::String(data["path"].to_string());
	ndata["h1"] 	= json::JsonValue::String(data["h1"].to_string());
	ndata["lang"] 	= json::JsonValue::String(data["lang"].to_string());

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
	add_to_set(&mut con, &"news".to_string(), &ndata.dump());
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
			    	// println!("nlu response data {}", value);

			    	let data = json::parse(&value);
			    	match data {
			    		Ok(json_data) => {
					    	// println!("nlu response json {:?}", json_data);
					    	process_nlu_data(json_data);
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
    delete_set(&mut con, "news".to_string());
    delete_set(&mut con, tgnews_nlu_reply.to_string());
    delete_set(&mut con, tgnews_nlu_request.to_string());

	let start_time 	= Utc::now();
    //SETUP DEBUG
	if query == "debug" {
	    println!("=============== RUNNING TGNEWS v0.4.2 ===============");
	    println!("=============== START TIME {} ===============", start_time);
	    println!("Searching for {}", query);
	    println!("In folder {}", filename);
	    block_on(run_nlu_service(&mut con, &query));
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
		print_news_start();
	    block_on(run_nlu_service(&mut con, &query));
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
		print_news_end();
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
	    	"path" => pstr,
	    	"lang" => key
	    };
	    con.publish(tgnews_nlu, lang_data.dump())?;
    }
    Ok(())

}

