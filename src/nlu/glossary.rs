extern crate redis;
use json::JsonValue;
use std::fs::{self, DirEntry, File};
use std::path::{PathBuf, Path};
use std::env;
use std::time;
use std::thread;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::collections::VecDeque;
use fuzzy_matcher::skim::{fuzzy_match, fuzzy_indices};
use std::sync::{Arc, Mutex};

use packer::Packer;

#[derive(Packer)]
#[packer(source = "glossary")]
struct Assets;

// Society (includes Politics, Elections, Legislation, Incidents, Crime)
// Economy (includes Markets, Finance, Business)
// Technology (includes Gadgets, Auto, Apps, Internet services)
// Sports (includes E-Sports)
// Entertainment (includes Movies, Music, Games, Books, Arts)
// Science (includes Health, Biology, Physics, Genetics)
// Other (news articles that don't fall into any of the above categories)

pub fn start(queue:Arc<Mutex<VecDeque<JsonValue>>>, offset:u64) {
    thread::spawn(move || {

		let sports:Vec<String> 	= load_sports_glossary();
		let games:Vec<String> 	= load_games_glossary();
		let corp:Vec<String> 	= load_corp_glossary();

		let mut not_finished = true;
		while not_finished {
		    let mut lock = queue.try_lock();
		    if let Ok(ref mut mtx) = lock {
		    	let item = mtx.pop_front();
			    drop(lock);
		    	if item != None {
			    	process_item(item.unwrap(), &sports, &games, &corp);
		    	}
		    } 
		    let _millis = time::Duration::from_millis(offset);
			thread::sleep(_millis);
			
		}
    });
}

fn process_item(item:JsonValue,sports:&Vec<String>, games:&Vec<String>, corp:&Vec<String>) {
	let h1 = &item["h1"].to_string();
	// println!("found item in queue {:?}", h1);
	let mut corp_score = 0;
	let mut sports_score = 0;
	for c in corp {
		let sc = fuzzy_indices(&h1, &c);
		if sc != None {
			let (score, indices) = sc.unwrap();
			// println!("score for {} in {}:{:?}",&c, &h1, score );
			if score > 0 {
				corp_score += score;
			}
		}
	}
	if corp_score > 80 {
		println!("corp score {} for {}", &corp_score, &h1);
	}
	for s in sports {
		let sc = fuzzy_indices(&h1, &s);
		if sc != None {
			let (score, indices) = sc.unwrap();
			if score > 0 {
				sports_score += score;
			}
		}
	}
	if sports_score > 80 {
		println!("sports score {} for {}", &sports_score, &h1);
	}
}

pub fn process_text(text: &str) {
	let (score, indices) = fuzzy_indices("axbycz", "abc").unwrap();
	assert_eq!(indices, [0, 2, 4]);
}

fn load_sports_glossary() -> Vec<String>  {
	let keys = [
		"glossary/sports/sports.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	let msports = data["glossary/sports/sports.json"]["sports"].members();
	for ms in msports {
		ndata.push(ms.to_string());
	}
	return ndata;
}

fn load_games_glossary() -> Vec<String> {
	let keys = [
		"glossary/games/steam.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	let msteam = data["glossary/games/steam.json"]["applist"]["apps"].members();
	for ms in msteam {
		ndata.push(ms["name"].to_string());
	}
	return ndata;
}

fn load_corp_glossary() -> Vec<String> {
	let keys = [
		"glossary/corporations/fortune500.json",
		"glossary/corporations/nasdaq.json",
		"glossary/corporations/newspapers.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	let mfortune = data["glossary/corporations/fortune500.json"]["companies"].members();
	let mnews = data["glossary/corporations/newspapers.json"]["newspapers"].members();
	let mnasdaq = data["glossary/corporations/nasdaq.json"]["corporations"].members();
	for mfs in mfortune {
		ndata.push(mfs.to_string());
	}
	for mnws in mnews {
		ndata.push(mnws.to_string());
	}
	for mnqs in mnasdaq {
		ndata.push(mnqs["symbol"].to_string());
		ndata.push(mnqs["name"].to_string());
	}
	return ndata;
}

fn get_required_assets(keys:Vec<&str>) -> json::JsonValue {
	let files = Assets::list();
	let mut data = json::JsonValue::new_object();
	for file in files {
		let required = keys.contains(&file);
		let raw = Assets::get_str(&file);
		
		if raw == None || !required {
			continue;
		}
	    let jdata = json::parse(raw.unwrap());
		match jdata {
			Ok(jresult) => {
				data[file] = jresult;
			},
			Err(e) => {

			}
		}
	}
	return data;

}
