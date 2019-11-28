extern crate whatlang;
extern crate select;
extern crate futures;
extern crate chrono;
extern crate redis;
extern crate json;
extern crate clap;

use chrono::{Utc};
use std::time::{Instant};
use std::str;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::env;
use futures::executor::block_on;
use redis::Commands;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use json::JsonValue;
use std::sync::mpsc::channel;
use std::thread;
use clap::{App, Arg};

mod printer;
mod nlu;
mod parser;

//REDIS KEYS
static tgnews_nlu_reply:&'static str = "tgnews_nlu_reply_list";
static tgnews_nlu_request:&'static str = "tgnews_nlu_request_list";
static tgnews_nlu:&'static str = "tgnews_nlu";
static tgnews_nlu_start:&'static str = "tgnews_nlu_start";
static tgnews_nlu_end:&'static str = "tgnews_nlu_end";
static tgnews_nlu_reply_timeout:usize = 5;

use printer::print_languages_start;
use printer::print_languages_end;
use printer::print_news_start;
use printer::print_news_end;
use nlu::run_nlu_service;
use nlu::wait_for_nlu_completion;
use nlu::process_nlu_data;
use nlu::run_nlu_listener;
use nlu::glossary;
use parser::parse_file;
use parser::visit_dirs;

fn main() {
	let bstart 									= Instant::now();
	let ru_db:Arc<Mutex<Vec<String>>> 			= Arc::new(Mutex::new(Vec::new()));
	let done_index:Arc<Mutex<Vec<String>>> 		= Arc::new(Mutex::new(Vec::new()));
	let category_db:Arc<Mutex<JsonValue>> 		= Arc::new(Mutex::new(json::JsonValue::new_object()));
	let gQueue:Arc<Mutex<VecDeque<JsonValue>>> 	= Arc::new(Mutex::new(VecDeque::new()));
    let args:Vec<String> 						= env::args().collect();
    let query	 								= &args[1];
    let filename 								= &args[2];
    let mut bduration 							= Instant::now().elapsed();
    let mut disable_python = true;
    let matches = App::new("TGNEWS")
        .args(&[
            Arg::with_name("query").index(1).help("options are: debug, news, categories, threads, top"),
            Arg::with_name("filename").index(2).help("directory path: ./DataClusteringSample0817/"),
            Arg::with_name("python")
                .help("enable python neural nets with --python")
                .long("python"),
            Arg::with_name("redis")
                .help(r#"enable redis cache with --redis="""#)
                .long("redis")
        ])
        .get_matches();
	if matches.is_present("python") {
        disable_python = false;
    } 
    //SETUP DEBUG
	if query == "debug" {
	    println!("=============== RUNNING TGNEWS v0.5.1 ===============");
	    println!("=============== START TIME {} ===============", Utc::now());
	    println!("Searching for {}", query);
	    println!("In folder {}", filename);
	    if disable_python == false {
		    run_nlu_service();
	    }
	    run_glossaries(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&category_db), disable_python);
	    println!("python && glosary setup done");
	    bduration = bstart.elapsed();
	    println!("total boot time: {:?}", bduration);
	}

	//SETUP LANGUAGES
	if query == "languages" {
		print_languages_start();
	}

	//SETUP NEWS
	if query == "news" {
		print_news_start();
		run_glossaries(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&category_db), disable_python);
	    if disable_python == false {
		    run_nlu_service();
	    }
	}
    

    //START DIRS
	let start = Instant::now();
    let path = Path::new(filename);
    let result = visit_dirs(path, Arc::clone(&gQueue), Arc::clone(&ru_db));
	

	if query == "languages" {
		print_languages_end(Arc::clone(&ru_db));
		return;
	}

	if query == "debug" {
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		block_on(wait_for_nlu_completion(Arc::clone(&gQueue), disable_python));
		//COUNT PERFORMANCE
		let end_time = Utc::now();
		let duration = start.elapsed();
	    println!("=============== ALL DONE! ===============");
	    println!("=============== END TIME {} ===============", end_time);
	    println!("=============== DURATION {:?} ===============", duration);
	    println!("=============== BDURATION {:?} ===============", bduration);
	    let mut lock1 = category_db.try_lock();
	    if let Ok(ref mut mtx) = lock1 {
		    // println!("all news data from glossary: {}", mtx.pretty(2));
		    println!("total news data from glossary: {:?}", mtx.len());
		}
	}
	//SETUP NEWS
	if query == "news" {
		block_on(wait_for_nlu_completion(Arc::clone(&gQueue), disable_python));
		print_news_end();
	}
}
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

fn run_glossaries(
	done_index:Arc<Mutex<Vec<String>>>, 
	gQueue:Arc<Mutex<VecDeque<JsonValue>>>,
	db:Arc<Mutex<JsonValue>>,
	disable_python:bool
) {
	if(!disable_python) {
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 11);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 12);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 13);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 14);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 15);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 16);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 21);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 22);
	}
	else {
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 11);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 12);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 13);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 14);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 15);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 16);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 21);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 22);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 23);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 33);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 37);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 61);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 53);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 27);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 34);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 88);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 23);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 55);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 51);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 52);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 54);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 56);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 58);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 88);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 24);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 26);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 27);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 29);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 30);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 61);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 62);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 63);

	}
}

