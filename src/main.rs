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
use std::sync::{Arc, Mutex,RwLock};
use std::collections::VecDeque;
use json::JsonValue;
use std::sync::mpsc::channel;
use std::collections::BTreeMap;
use std::thread;
use clap::{App, Arg};

#[macro_use]
extern crate tantivy;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tempdir::TempDir;
use threadpool::ThreadPool;

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
use nlu::wait_for_nlu_completion_minimal;
use nlu::process_nlu_data;
use nlu::run_nlu_listener;
use nlu::glossary;
use parser::parse_file;
use parser::visit_dirs;

fn main() {
	let index_path 								= TempDir::new("temp").unwrap();
	let bstart 									= Instant::now();
	let ru_db:Arc<Mutex<Vec<String>>> 			= Arc::new(Mutex::new(Vec::new()));
	let done_index:Arc<Mutex<Vec<String>>> 		= Arc::new(Mutex::new(Vec::new()));
	let category_db:Arc<Mutex<JsonValue>> 		= Arc::new(Mutex::new(json::JsonValue::new_object()));
	let gQueue:Arc<Mutex<VecDeque<JsonValue>>> 	= Arc::new(Mutex::new(VecDeque::new()));
	let names_db:Arc<Mutex<BTreeMap<String, String>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let args:Vec<String> 						= env::args().collect();
    let fs_pool 								= ThreadPool::with_name("fs_pool".into(), 200);
    let ws_pool 								= ThreadPool::with_name("ws_pool".into(), 1);
    let query	 								= &args[1];
    let filename 								= &args[2];
    let mut bduration 							= Instant::now().elapsed();
    let mut disable_python 						= true;
    let matches = App::new("TGNEWS")
        .args(&[
            Arg::with_name("query").index(1).help("options are: debug, news, categories, threads, top"),
            Arg::with_name("filename").index(2).help("directory path: ./DataClusteringSample0817/"),
            Arg::with_name("python")
                .help(r#"enable python neural nets with --python, requires redis running on redis://127.0.0.1"#)
                .long("python")
        ])
        .get_matches();
	if matches.is_present("python") {
        disable_python = false;
    } 

	let mut schema_builder = Schema::builder();
			schema_builder.add_text_field("title", TEXT | STORED);
			schema_builder.add_text_field("body", TEXT | STORED);
    let schema 	= schema_builder.build();
    let mut index = Index::create_in_ram(schema.clone());
    	&index.set_default_multithread_executor();
    let index_writer = Arc::new(RwLock::new(index.writer(100_000_000).unwrap()));
    //SETUP DEBUG
	if query == "debug" {

	    println!("=============== RUNNING TGNEWS v0.7.1 ===============");
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
    let result = visit_dirs(
    	path, 
    	Arc::clone(&gQueue), 
    	Arc::clone(&ru_db), 
    	Arc::clone(&names_db), 
    	Arc::clone(&index_writer), 
    	schema.clone(), 
    	fs_pool.clone(), 
    	ws_pool.clone(), 
    	disable_python
    );
	fs_pool.join();
	drop(fs_pool);
	if query == "debug" {
		println!("====================== FileSystem FINISHED IN {:?} ======================", start.elapsed());
		println!("total jobs for ws {}", ws_pool.queued_count());
	}
	ws_pool.join();
	let mut index_writer_wlock = index_writer.write().unwrap();
            index_writer_wlock.commit().unwrap();
	    drop(index_writer_wlock);

	if query == "languages" {
		print_languages_end(Arc::clone(&ru_db));
		return;
	}

	if query == "debug" {
		println!("====================== WriteSystem FINISHED IN {:?} ======================", start.elapsed());
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
		println!("====================== WAITING FOR NLU COMPLETION ======================");
	    let mut lock0 = names_db.try_lock();
	    if let Ok(ref mut mtx) = lock0 {
		    println!("total items in names : {:?}", mtx.len());
		}
		drop(lock0);
		let _index = Arc::new(Mutex::new(index));
		let bq_service = glossary::start_bigquery_service(
			Arc::clone(&_index), 
			Arc::clone(&category_db), 
			schema.clone()
		);
		println!("====================== BTREE FINISHED IN {:?} ======================", start.elapsed());
		println!("====================== FINISHED BTREE ======================");
		println!("====================== FINISHED BTREE ======================");
		println!("====================== FINISHED BTREE ======================");
		println!("====================== FINISHED BTREE ======================");
		if !disable_python {
			block_on(wait_for_nlu_completion(Arc::clone(&gQueue), disable_python));
		}
		else {
			block_on(wait_for_nlu_completion_minimal(Arc::clone(&gQueue), disable_python));
		}
		//COUNT PERFORMANCE
		let end_time = Utc::now();
		let duration = start.elapsed();
	    println!("=============== ALL DONE! ===============");
	    println!("=============== END TIME {} ===============", end_time);
	    println!("=============== DURATION {:?} ===============", duration);
	    println!("=============== BDURATION {:?} ===============", bduration);
	    let mut lock1 = category_db.try_lock();
	    if let Ok(ref mut mtx) = lock1 {
		    println!("total news data from glossary: {:?}", mtx.len());
		}
		drop(lock1);
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
		// glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 1);
		// glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 2);
		// glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 3);
		// glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 4);
		// glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 5);
		glossary::start(Arc::clone(&done_index), Arc::clone(&gQueue), Arc::clone(&db), 11);
}

