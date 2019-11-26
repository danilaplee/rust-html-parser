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

fn main() {
	let bstart = Instant::now();
    let args: Vec<String> = env::args().collect();
    let query	 = &args[1];
    let filename = &args[2];
    let mut bduration = Instant::now().elapsed();
	
    //CLEAN DB SYNC
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let p1: () = con.publish(tgnews_nlu, "done").unwrap();
    delete_set(&mut con, "eng".to_string());
    delete_set(&mut con, "rus".to_string());
    delete_set(&mut con, "news".to_string());
    delete_set(&mut con, tgnews_nlu_reply.to_string());
    delete_set(&mut con, tgnews_nlu_request.to_string());

    //SETUP DEBUG
	if query == "debug" {
	    println!("=============== RUNNING TGNEWS v0.4.2 ===============");
	    println!("=============== START TIME {} ===============", Utc::now());
	    println!("Searching for {}", query);
	    println!("In folder {}", filename);
	    block_on(run_nlu_service(&mut con, &query));
	    println!("python setup done");
	    run_nlu_listener();
	    println!("listener setup done");
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
	    block_on(run_nlu_service(&mut con, &query));
	    run_nlu_listener();
	}
    

    //START DIRS
	let start = Instant::now();
    let path = Path::new(filename);
    let result = visit_dirs(path);
	

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
	    println!("=============== BDURATION {:?} ===============", bduration);
	}
	//SETUP NEWS
	if query == "news" {
		block_on(wait_for_nlu_completion(&mut con));
		print_news_end();
	}
    let p2:() = con.publish(tgnews_nlu, "done").unwrap();

}

