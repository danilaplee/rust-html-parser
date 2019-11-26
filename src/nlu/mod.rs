extern crate redis;
use redis::Commands;
use redis::RedisResult;
use std::collections::HashSet;
use json::object;
use json::JsonValue;
use std::fs::File;
use std::env;
use std::thread;
use std::io;
use std::process::{Command, Stdio};
use path_clean::PathClean;
use std::path::{PathBuf, Path};
use super::tgnews_nlu_reply;
use super::tgnews_nlu_request;
use super::tgnews_nlu;
use super::tgnews_nlu_start;
use super::tgnews_nlu_end;
use super::tgnews_nlu_reply_timeout;
use super::add_to_set;

pub mod glossary;

pub fn absolute_path<P>(path: P) -> io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf().clean())
    } else {
        Ok(env::current_dir()?.join(path).clean())
    }
}

pub async fn run_nlu_service(con: &mut redis::Connection, query: &str) -> Result<(), Box<dyn std::error::Error + 'static>>   {

	let mut pubsub = con.as_pubsub();
    let f = absolute_path(PathBuf::from("./python/nlu_service.py")).unwrap().into_os_string();
	pubsub.subscribe(tgnews_nlu_start);
	if query == "debug" {
		let python_process = Command::new("python3")
		.arg(f)
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
		.spawn()
		.expect("failed to execute process");
		println!("subscribed to tgnews_nlu");
	}
	else {
		let python_process = Command::new("python3")
		.arg(f)
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

pub async fn wait_for_nlu_completion(con: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error + 'static>>   {

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

pub fn run_nlu_listener() -> Result<(), Box<dyn std::error::Error + 'static>> {
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

pub fn process_nlu_data(data:JsonValue) {
	
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

	let mut words_map 	= json::JsonValue::new_object();
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
			let mut kk = ""; 
			if s == gpe_key {
				gpe = true;
				kk = "gpe";
			}
			if s == org_key {
				org = true;
				kk = "org";
			}
			if s == work_of_art_key  {
				work_of_art = true;
				kk = "art";
			}
			if s == person_key  {
				person = true;
				kk = "person";
			}
			if s == product_key  {
				product = true;
				kk = "product";
			}
			if s == money_key  {
				money = true;
				kk = "money";
			}
			if s == loc_key  {
				location = true;
				kk = "loc";
			}
			if words_map[kk].is_null() {
				words_map[kk] = json::JsonValue::new_array();
			}
			words_map[kk].push(words[current].to_string());
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
			println!("news worthy: {:?}, org {}, gpe {}, person {}, money {}, work_of_art {}", data["h1"].to_string(), org, gpe, person, money, work_of_art);
		}
		if query == "news" {
			println!(r#"			{:?},"#, &data["path"].to_string());
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
		ndata["words"]	= words_map;

	    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
	    let mut con = client.get_connection().unwrap();
		add_to_set(&mut con, &"news".to_string(), &ndata.dump());
	}
}
