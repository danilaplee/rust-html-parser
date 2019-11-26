extern crate redis;
use redis::Commands;
use redis::RedisResult;
use std::collections::HashSet;
use json::object;
use json::JsonValue;
use std::fs::{self, DirEntry, File};
use std::path::{PathBuf, Path};
use std::env;
use std::thread;
use std::io::BufReader;
use std::process::{Command, Stdio};
use super::tgnews_nlu_reply;
use super::tgnews_nlu_request;
use super::tgnews_nlu;
use super::tgnews_nlu_start;
use super::tgnews_nlu_end;
use super::tgnews_nlu_reply_timeout;
use super::add_to_set;

// Society (includes Politics, Elections, Legislation, Incidents, Crime)
// Economy (includes Markets, Finance, Business)
// Technology (includes Gadgets, Auto, Apps, Internet services)
// Sports (includes E-Sports)
// Entertainment (includes Movies, Music, Games, Books, Arts)
// Science (includes Health, Biology, Physics, Genetics)
// Other (news articles that don't fall into any of the above categories)

pub fn start() {
	let sports:JsonValue = load_sports_glossary();
	let games:JsonValue = load_games_glossary();
}

pub fn process_word() {

}

fn load_sports_glossary() -> json::JsonValue  {
	let data = load_glossary("sports");
	return data;
}

fn load_games_glossary() -> json::JsonValue {
	let data = load_glossary("games");
	return data;
}

fn aggregate_glossary(entry: &DirEntry) -> json::JsonValue {
    let path = entry.path();
    let data = fs::read_to_string(&path);
    match data {
    	Ok(raw) => {
		    let jdata = json::parse(&raw);
    		match jdata {
    			Ok(jresult) => {
    				return jresult;
    			},
    			Err(e) => {

    			}
    		}
    	},
    	Err(e) => {

    	}
    }
	return json::JsonValue::new_object();
}

fn load_glossary(glossary_type:&str) -> json::JsonValue {
		let dir = PathBuf::from(format!("./glossary/{}",glossary_type));
		let mut entries = json::JsonValue::new_array();
        for entry in fs::read_dir(dir).unwrap() {
        	let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
            	for k in fs::read_dir(path).unwrap() {
            		entries.push(aggregate_glossary(&k.unwrap()));
            	}
            }
	        else {
	        	entries.push(aggregate_glossary(&entry));
	        }
        }
        return entries;
}