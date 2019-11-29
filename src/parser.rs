extern crate whatlang;
extern crate select;
extern crate futures;
extern crate chrono;
extern crate redis;
extern crate json;
extern crate threadpool;

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
use std::collections::BTreeMap;
use futures::join;
use tantivy::{IndexWriter, doc};
use tantivy::schema::*;
use threadpool::ThreadPool;

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
	ru_db:Arc<Mutex<Vec<String>>>,
	names_db:Arc<Mutex<BTreeMap<String, String>>>,
	_index:Arc<Mutex<IndexWriter>>,
	_schema:Schema,
	_pool:ThreadPool,
	ipool:ThreadPool,
	ipool2:ThreadPool,
	_index2:Arc<Mutex<IndexWriter>>,
	ipool3:ThreadPool,
	_index3:Arc<Mutex<IndexWriter>>
	) -> io::Result<()> {
    let mut ittr = 0;
    let data = json::JsonValue::new_array();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
	    	ittr += 1;
            let entry = entry?;

            let path 	= entry.path();
		    let q 		= Arc::clone(&queue);
		    let rus 	= Arc::clone(&ru_db);
		    let names 	= Arc::clone(&names_db);
		    let schema 	= _schema.clone();
		    let p 		= _pool.clone();
		    let ip 		= ipool.clone();
		    let index 	= Arc::clone(&_index);
		    let ip2 	= ipool2.clone();
		    let index2 	= Arc::clone(&_index2);
		    let ip3 	= ipool3.clone();
		    let index3 	= Arc::clone(&_index3);

            if path.is_dir() {
			    _pool.execute(move || {
	            	visit_dirs(&path, q, rus, names, index, schema, p, ip, ip2, index2, ip3, index3);
			    });
            } else {
			    let args: Vec<String> = env::args().collect();
			    let query = &args[1];;
			    let mut cp 	= ThreadPool::new(1);
			    let mut ci 	= Arc::clone(&_index);
			    if ip.queued_count() < 20000 {
			    	cp = ip.clone();
			    	ci = index.clone();
			    }
			    else {
			    	if ip2.queued_count() < 30000 {
				    	cp = ip2.clone();
				    	ci = index2.clone();
			    	}
			    	else {
				    	cp = ip3.clone();
				    	ci = index3.clone();
			    	}
			    }
	            match parse_file(&entry, q, rus, names, ci, schema, cp) {
		            Result::Ok(val) => val,
		            Result::Err(err) => (),
	            }
            }
        }
    }
    Ok(())
}


pub fn parse_file(entry: &DirEntry, 
	queue:Arc<Mutex<VecDeque<JsonValue>>>,
	ru_db:Arc<Mutex<Vec<String>>>,
	names_db:Arc<Mutex<BTreeMap<String, String>>>,
	_index:Arc<Mutex<IndexWriter>>,
	schema:Schema,
	_pool:ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {

    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    let path = entry.path();
    let pstr:String = String::from(path.as_path().to_str().unwrap());
    let psss = pstr.to_string();
	
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
    let spa = info.lang() == Lang::Spa;
    let por = info.lang() == Lang::Por;
    let ita = info.lang() == Lang::Ita;
    let fra = info.lang() == Lang::Fra;
    let ukr = info.lang() == Lang::Ukr;
    let bel = info.lang() == Lang::Bel;

    if (eng || rus) && !(spa || por || ita || fra || bel || ukr) {
	    let mut key = "rus";
	    if eng {
	    	key = "eng";

			if query == "debug" {
		    	// println!("saving file: {}", &h1);
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

	    _pool.execute(move || {
		    // REDIS DISABLED TEMPORARY
		    // let client = redis::Client::open("redis://127.0.0.1/")?;
		    // let mut con = client.get_connection()?;
		    // con.lpush(tgnews_nlu, &lang_data.dump())?;

		    let mut lock = queue.try_lock();
		    if let Ok(ref mut mtx) = lock {
		        // println!("total queue length: {:?}", mtx.len());
		       	mtx.push_back(json::parse(&lang_data.dump()).unwrap());
		    } else {
		        // println!("parser 1 try_lock failed");
		    }
		    drop(lock);
		    if key == "rus" {
			    let mut lock2 = ru_db.try_lock();
			    if let Ok(ref mut mtx2) = lock2 {
			        // println!("total queue length: {:?}", mtx.len());
			       	mtx2.push(pstr);
			    } else {
			        // println!("parser 2 try_lock failed");
			    }
			    drop(lock2);
		    }

		    let mut lock3 = names_db.try_lock();
		    if let Ok(ref mut mtx3) = lock3 {
		        // println!("total queue length: {:?}", mtx.len())
		        let h2 = &lang_data["h1"];
		        let h3 = &lang_data["h1"];
		       	mtx3.insert(h2.to_string().to_lowercase(), h3.to_string().to_lowercase());
		    } else {
		        // println!("parser 3 try_lock failed");
		    }
		    drop(lock3);

	        let title = schema.get_field("title").unwrap();
		    let body = schema.get_field("body").unwrap();
	        let h4 = &lang_data["h1"];
	        let mut write4 = false;
	        while write4 == false {
			    let mut lock4 = _index.try_lock();
			    if let Ok(ref mut writer) = lock4 {
			        // println!("parser 4 try_lock success");
			    	writer.add_document(doc!(
					    title => h4.to_string(),
					    body => ""
				    ));
				    writer.commit();
				    write4 = true;
				    drop(lock4);
			    }
			    else {
				    drop(lock4);
					thread::sleep(time::Duration::from_nanos(1));
			    }
	        }
	    });
	    // println!("total size of queue: {:?}", queue.add_work(&lang_data));
    }
    Ok(())

}
