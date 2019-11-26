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
		let science:Vec<String> = load_science_glossary();
		let medicine:Vec<String>= load_medicine_glossary();

		let mut not_finished = true;
		while not_finished {
		    let mut lock = queue.try_lock();
		    if let Ok(ref mut mtx) = lock {
		    	let item = mtx.pop_front();
			    drop(lock);
		    	if item != None {
			    	process_item(&item.unwrap(), &sports, &games, &corp, &medicine, &science);
		    	}
		    } 
		    let _millis = time::Duration::from_millis(offset);
			thread::sleep(_millis);
			
		}
    });
}

fn process_item(
	item:&JsonValue,
	sports:&Vec<String>, 
	games:&Vec<String>, 
	corp:&Vec<String>, 
	medicine:&Vec<String>,
	science:&Vec<String>) {
	
	// println!("found item in queue {:?}", h1);
	let h1 = &item["h1"].to_string();
	let corp_score = find_theme_score(&item, corp);
	let sports_score = find_theme_score(&item, sports);
	let medicine_score = find_theme_score(&item, medicine);
	let science_score = find_theme_score(&item, science);
	if corp_score > 80 || sports_score > 80 || medicine_score > 80
	|| science_score > 80 {
		println!("glossary news: {:?}", &h1);
		println!("corp score {}", &corp_score);
		println!("sports score {}", &sports_score);
		println!("medicine score {}", &medicine_score);
		println!("science score {}", &science_score);
	}
}

fn find_theme_score(item:&JsonValue,theme:&Vec<String>) -> i64 {

	let mut _score = 0;
	let h1 = &item["h1"].to_string();
	for c in theme {
		let sc = fuzzy_indices(&h1, &c);
		if sc != None {
			let (score, indices) = sc.unwrap();
			// println!("score for {} in {}:{:?}",&c, &h1, score );
			if score > 0 {
				_score += score;
			}
		}
	}
	return _score;
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

fn load_science_glossary() -> Vec<String> {
	let keys = [
		"glossary/science/planets.json",
		"glossary/science/minor_planets.json",
		"glossary/science/elements.json",
		"glossary/science/weather_conditions.json",
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	let mweather = data["glossary/science/weather_conditions.json"]["conditions"].members();
	for m in mweather {
		ndata.push(m.to_string());
	}
	let mmplanets = data["glossary/science/minor_planets.json"]["minor_planets"].members();
	for m in mmplanets {
		ndata.push(m.to_string());
	}
	let mplanets = data["glossary/science/planets.json"]["planets"].members();
	for m in mplanets {
		ndata.push(m["name"].to_string());
	}
	let melements = data["glossary/science/elements.json"]["elements"].members();
	for m in melements {
		ndata.push(m["name"].to_string());
		ndata.push(m["discoverer"].to_string());
	}
	return ndata;
}

fn load_medicine_glossary() -> Vec<String> {
	let keys = [
		"glossary/medicine/drugNameStems.json",
		"glossary/medicine/cancer.json",
		"glossary/medicine/hospitals.json",
		"glossary/medicine/infectious_diseases.json",
		"glossary/medicine/diagnoses.json",
		"glossary/medicine/symptoms.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	for ms in data["glossary/medicine/drugNameStems.json"]["stems"].members() {
		ndata.push(ms.to_string());
	}
	for ms in data["glossary/medicine/cancer.json"]["cancers"].members() {
		ndata.push(ms.to_string());
	}
	for ms in data["glossary/medicine/hospitals.json"]["hospitals"].members() {
		ndata.push(ms.to_string());
	}
	for ms in data["glossary/medicine/symptoms.json"]["diagnoses"].members() {
		ndata.push(ms["desc"].to_string());
	}
	for ms in data["glossary/medicine/symptoms.json"]["symptoms"].members() {
		ndata.push(ms.to_string());
	}
	for ms in data["glossary/medicine/infectious_diseases.json"]["diseases"].members() {
		ndata.push(ms.to_string());
	}
	for ms in data["glossary/medicine/diseases.json"]["diseases"].members() {
		for nn in ms.members() {
			ndata.push(nn.to_string())
		}
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
		"glossary/corporations/newspapers.json",
		"glossary/corporations/dija.json",
		"glossary/humans/richpeople.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	for m in data["glossary/corporations/fortune500.json"]["companies"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/corporations/newspapers.json"]["newspapers"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/corporations/nasdaq.json"]["corporations"].members() {
		ndata.push(m["symbol"].to_string());
		ndata.push(m["name"].to_string());
	}
	for m in data["glossary/corporations/dija.json"]["corporations"].members() {
		ndata.push(m["name"].to_string());
	}
	for m in data["glossary/humans/richpeople.json"]["richPeople"].members(){
		ndata.push(m["symbol"].to_string());
		ndata.push(m["name"].to_string());
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
