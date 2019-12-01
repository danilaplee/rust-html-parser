extern crate whatlang;
extern crate select;
extern crate futures;
extern crate chrono;
extern crate redis;
extern crate json;
extern crate threadpool;
extern crate regex;

use regex::Regex;
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
use std::sync::{Arc, Mutex, RwLock};
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
	_index:Arc<RwLock<IndexWriter>>,
	_schema:Schema,
	_pool:ThreadPool,
	ipool:ThreadPool,
	python_disabled:bool,
	) -> io::Result<()> {
    let mut ittr = 0;
    let data 	= json::JsonValue::new_array();
    let mut conn:Arc<Mutex<Option<redis::Connection>>> = Arc::new(Mutex::new(None));
    if !python_disabled {
	    let client 	= redis::Client::open("redis://0.0.0.0/").unwrap();
	    let con 	= client.get_connection();
			conn 	= Arc::new(Mutex::new(Some(con.unwrap())));
    }
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

            if path.is_dir() {
			    _pool.execute(move || {
	            	visit_dirs(&path, q, rus, names, index, schema, p, ip, python_disabled);
			    });
            } else {
			    let args: Vec<String> = env::args().collect();
			    let query = &args[1];;
	            match parse_file(
	            	&entry, 
	            	q, 
	            	rus, 
	            	names, 
	            	index, 
	            	schema, 
	            	ip, 
	            	python_disabled, 
	            	conn.clone()
	            ) {
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
	_index:Arc<RwLock<IndexWriter>>,
	schema:Schema,
	_pool:ThreadPool,
	python_disabled:bool,
	redis_conn:Arc<Mutex<Option<redis::Connection>>>) -> Result<(), Box<dyn std::error::Error + 'static>> {

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
    let epo = info.lang() == Lang::Epo;
    let deu = info.lang() == Lang::Deu;
    let arb = info.lang() == Lang::Arb;
    let hin = info.lang() == Lang::Hin;
    let kat = info.lang() == Lang::Kat;
    let jpn = info.lang() == Lang::Jpn;
    let ces = info.lang() == Lang::Ces;
    let ind = info.lang() == Lang::Ind;
    let pan = info.lang() == Lang::Pan;
    let tha = info.lang() == Lang::Tha;
    let lav = info.lang() == Lang::Lav;
    let est = info.lang() == Lang::Est;

    if (eng || rus) && !(spa || por || ita || fra || bel 
    	|| ukr || deu || epo || arb || hin || kat || jpn 
    	|| ces || ind || pan || tha || lav || est) {
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
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		        let h2 = re.replace_all(&lang_data["h1"].to_string(), "").to_string();
		        let h3 = re.replace_all(&lang_data["h1"].to_string(), "").to_string();
		       	mtx3.insert(h2.to_string(), h3.to_string());
		    } else {
		        // println!("parser 3 try_lock failed");
		    }
		    drop(lock3);

	        let title = schema.get_field("title").unwrap();
		    let body = schema.get_field("body").unwrap();
	        let h4 = &lang_data["h1"];
	        let mut write4 = false;
	        while write4 == false {
			    let lock4 = _index.read();
			    if let Ok(ref writer) = lock4 {

					let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
			    	writer.add_document(doc!(
					    title => re.replace_all(&h4.to_string(), "").to_string(),
					    body => lang_data["path"].to_string()
				    ));
				    write4 = true;
				    drop(lock4);
			    }
			    else {
				    drop(lock4);
					thread::sleep(time::Duration::from_nanos(1));
			    }
	        }
		    if !python_disabled {
				let mut rlock = redis_conn.lock().unwrap();
				let con:&mut redis::Connection = rlock.as_mut().unwrap();
				let _ : () = con.lpush(tgnews_nlu, &lang_data.dump()).unwrap();
			    drop(rlock);
		    }
	    });
    }
    Ok(())

}
