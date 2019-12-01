extern crate regex;

use json::JsonValue;
use json::object;
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
use std::collections::BTreeMap;
use regex::Regex;


use packer::Packer;

#[derive(Packer)]
#[packer(source = "glossary")]
struct Assets;

fn is_double_name(input:String) -> bool {
	let items = input.split_whitespace();
	let len:Vec<&str> = input.split_whitespace().collect();
	for i in items {
		let i1 = format!("{} ", &i);
		let i2 = format!(" {}", &i);
		// println!("looking for {:?}", (&i1,&i2));
		if (len.len() > (input.len()/2)) || (input.matches(&i).count() > 1 && input.matches(&i1).count() > 1)
		|| (input.matches(&i).count() > 1 && input.matches(&i2).count() > 1) {
			// println!("double name: {}", input);
			return true;
		}
	}
	return false;
}

pub fn load_sports_glossary() -> Vec<String>  {
	let keys = [
		"glossary/sports/sports.json",
		"glossary/sports/nhl_teams.json",
		"glossary/sports/nfl_teams.json",
		"glossary/sports/nba_teams.json",
		"glossary/sports/mlb_teams.json",
		"glossary/sports/football/epl_teams.json",
		"glossary/sports/football/laliga_teams.json",
		"glossary/sports/football/serieA.json",
		"glossary/people/wrestlers.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	for m in data["glossary/sports/sports.json"]["sports"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/people/wrestlers.json"]["wrestlers"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/sports/nhl_teams.json"]["nhl_teams"].members() {
		ndata.push(m["name"].to_string());
		ndata.push(m["stadium"].to_string());
	}
	for m in data["glossary/sports/nfl_teams.json"]["nfl_teams"].members() {
		ndata.push(m["name"].to_string());
		ndata.push(m["stadium"].to_string());
	}
	for m in data["glossary/sports/nba_teams.json"]["nba_teams"].members() {
		ndata.push(m["name"].to_string());
		ndata.push(m["stadium"].to_string());
	}
	for m in data["glossary/sports/mlb_teams.json"]["mlb_teams"].members() {
		ndata.push(m["name"].to_string());
		ndata.push(m["stadium"].to_string());
	}
	for m in data["glossary/sports/football/epl_teams.json"]["epl_teams"].members() {
		ndata.push(m["name"].to_string());
		ndata.push(m["stadium"].to_string());
		ndata.push(m["manager"].to_string());
	}
	for m in data["glossary/sports/football/laliga_teams.json"]["laliga_teams"].members() {
		ndata.push(m["name"].to_string());
		ndata.push(m["stadium"].to_string());
		ndata.push(m["manager"].to_string());
	}
	
	return ndata;
}

pub fn load_science_glossary() -> Vec<String> {
	let keys = [
		"glossary/science/planets.json",
		"glossary/science/minor_planets.json",
		"glossary/science/elements.json",
		"glossary/science/weather_conditions.json",
		"glossary/humans/scientists.json",
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
	for ms in data["glossary/humans/scientists.json"]["humans"].members() {
		ndata.push(ms.to_string());
	}
	return ndata;
}

pub fn load_medicine_glossary() -> Vec<String> {
	let keys = [
		"glossary/medicine/drugNameStems.json",
		"glossary/medicine/cancer.json",
		"glossary/medicine/hospitals.json",
		"glossary/medicine/infectious_diseases.json",
		"glossary/medicine/diagnoses.json",
		"glossary/medicine/symptoms.json",
		"glossary/humans/bodyParts.json"
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
	for ms in data["glossary/humans/bodyParts.json"]["bodyParts"].members() {
		ndata.push(ms.to_string());
	}
	for ms in data["glossary/medicine/diseases.json"]["diseases"].members() {
		for nn in ms.members() {
			ndata.push(nn.to_string())
		}
	}
	return ndata;
}

pub fn load_games_glossary() -> Vec<String> {
	let keys = [
		"glossary/games/steam.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	let msteam = data["glossary/games/steam.json"]["applist"]["apps"].members();
	for ms in msteam {
		let name = ms["name"].to_string();
		let splt: Vec<&str> = name.split_whitespace().collect();
		if (&name.len() < &12) 
		|| is_double_name(name.to_lowercase())
		|| (name.to_lowercase().contains("the") && splt.len() > 4)
		|| (name.to_lowercase().contains("years") && splt.len() > 4)
		|| name.to_lowercase().contains("hong kong") 
		|| name.to_lowercase().contains("world war") 
		|| name.to_lowercase() == "death"
		|| name.to_lowercase() == "death toll"
		|| name.to_lowercase() == "human rights"
		|| name.to_lowercase() == "cannabis"
		|| name.to_lowercase() == "one night"
		|| name.to_lowercase() == "love love love"
		|| name == "Run Zeus Run"
		|| name == "Bump Bump Bump"
		|| name == "Beat Da Beat"
		|| name == "I L L U S I O N"
		|| name == "All You Can Eat"
		|| name == "Combat Force"
		|| name == "KILL la KILL -IF"
		|| name == "Hentai 2+2=4"
		|| name == "Door To Door"
		|| ndata.contains(&name) {
			continue;
		}
		let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		// let re = Regex::new(r#"[^a-zA-Z0-9]+"#).unwrap();
		ndata.push(re.replace_all(&ms["name"].to_string(), "").to_string());
	}
	return ndata;
}
pub fn load_games_btree() -> BTreeMap<String, String> {
	let keys = [
		"glossary/games/steam.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: BTreeMap<String, String> = BTreeMap::new();
	let msteam = data["glossary/games/steam.json"]["applist"]["apps"].members();
	for ms in msteam {
		let name = ms["name"].to_string().to_lowercase();
		if (&name.len() < &7) 
		|| !name.contains(" ") 
		|| (name.contains("the") && name.len() <= 8)
		|| (name.contains("years") && name.len() <= 8)
		|| name.contains("hong kong") 
		|| name.contains("world war") 
		|| name == "death"
		|| name == "death toll"
		|| name == "human rights"
		|| name == "cannabis"
		|| name == "one night" {
			continue;
		}
		ndata.insert(ms["name"].to_string().to_lowercase(), name);
	}
	return ndata;
}

pub fn load_corp_glossary() -> Vec<String> {
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
pub fn load_tech_glossary() -> Vec<String> {
	let keys = [
		"glossary/technology/appliances.json",
		"glossary/technology/computer_sciences.json",
		"glossary/technology/new_technologies.json",
		"glossary/technology/social_networking_websites.json",
		"glossary/technology/video_hosting_websites.json",
		"glossary/technology/photo_sharing_websites.json",
		"glossary/corporations/cars.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for m in data["glossary/technology/appliances.json"]["appliances"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/technology/computer_sciences.json"]["computer_sciences"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/technology/new_technologies.json"]["technologies"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/technology/social_networking_websites.json"]["socialNetworkingWebsites"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/technology/video_hosting_websites.json"]["videoHostingWebsites"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/technology/photo_sharing_websites.json"]["PhotoSharingWebsites"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/corporations/cars.json"]["cars"].members() {
		ndata.push(m.to_string());
	}
	return ndata;
}

pub fn load_music_glossary() -> Vec<String> {
	let keys = [
		"glossary/music/female_classical_guitarists.json",
		"glossary/music/instruments.json",
		"glossary/music/rock_hall_of_fame.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for m in data["glossary/music/female_classical_guitarists.json"]["data"].members() {
		ndata.push(m["name"].to_string());
	}
	for m in data["glossary/music/instruments.json"]["instruments"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/music/rock_hall_of_fame.json"]["artists"].members() {
		ndata.push(m["name"].to_string());
	}
	return ndata;
}


pub fn load_book_glossary() -> Vec<String> {
	let keys = [
		"glossary/books/bestsellers.json",
		"glossary/humans/authors.json",
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for m in data["glossary/books/bestsellers.json"]["books"].members() {
		ndata.push(m["title"].to_string());
	}
	for m in data["glossary/humans/authors.json"]["authors"].members() {
		ndata.push(m.to_string());
	}
	return ndata;
}

pub fn load_art_glossary() -> Vec<String> {
	let keys = [
		"glossary/art/isms.json",
		"glossary/objects/clothing.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for m in data["glossary/art/isms.json"]["isms"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/objects/clothing.json"]["clothing"].members() {
		ndata.push(m.to_string());
	}
	return ndata;
}
pub fn load_terror_glossary() -> Vec<String> {
	let aus 	= "glossary/societies_and_groups/designated_terrorist_groups/australia.json";
	let can 	= "glossary/societies_and_groups/designated_terrorist_groups/canada.json";
	let china 	= "glossary/societies_and_groups/designated_terrorist_groups/china.json";
	let egypt 	= "glossary/societies_and_groups/designated_terrorist_groups/egypt.json";
	let eu 		= "glossary/societies_and_groups/designated_terrorist_groups/european_union.json";
	let india 	= "glossary/societies_and_groups/designated_terrorist_groups/india.json";
	let iran	= "glossary/societies_and_groups/designated_terrorist_groups/iran.json";
	let israel	= "glossary/societies_and_groups/designated_terrorist_groups/israel.json";
	let kzh		= "glossary/societies_and_groups/designated_terrorist_groups/kazakhstan.json";
	let rus		= "glossary/societies_and_groups/designated_terrorist_groups/russia.json";
	let sa		= "glossary/societies_and_groups/designated_terrorist_groups/saudi_arabia.json";
	let tunisia	= "glossary/societies_and_groups/designated_terrorist_groups/tunisia.json";
	let turkey	= "glossary/societies_and_groups/designated_terrorist_groups/turkey.json";
	let ukr		= "glossary/societies_and_groups/designated_terrorist_groups/ukr.json";
	let uab		= "glossary/societies_and_groups/designated_terrorist_groups/united_arab_emirates.json";
	let uk		= "glossary/societies_and_groups/designated_terrorist_groups/united_kingdom.json";
	let us		= "glossary/societies_and_groups/designated_terrorist_groups/united_states.json";
	let un		= "glossary/societies_and_groups/designated_terrorist_groups/united_nations.json";

	let keys = [
		aus, can, china, egypt, eu, india, israel, iran, kzh, sa, rus, tunisia, turkey, ukr, uab, uk, us, un
	].to_vec();

	let keys2 = [
		aus, can, china, egypt, eu, india, israel, iran, kzh, sa, rus, tunisia, turkey, ukr, uab, uk, us, un
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for k in keys2 {
		for m in data[k].members() {
			ndata.push(m.to_string());
		}
	}
	return ndata;
}

pub fn load_etv_glossary() -> Vec<String> {
	let keys = [
		"glossary/film-tv/tv_shows.json",
		"glossary/film-tv/popular-movies.json",
		"glossary/humans/celebrities.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for m in data["glossary/film-tv/tv_shows.json"]["tv_shows"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/film-tv/popular-movies.json"]["popular-movies"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/humans/celebrities.json"]["celebrities"].members() {
		ndata.push(m.to_string());
	}
	return ndata;
}

pub fn load_ops_glossary() -> Vec<String> {
	let keys = [
		"glossary/governments/mass-surveillance-project-names.json",
		"glossary/governments/nsa_projects.json",
		"glossary/governments/us_mil_operations.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();

	for m in data["glossary/governments/us_mil_operations.json"]["operations"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/governments/nsa_projects.json"]["codenames"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/governments/mass-surveillance-project-names.json"]["projects"].members() {
		ndata.push(m["name"].to_string());
	}
	return ndata;

}

pub fn load_gov_glossary() -> Vec<String> {
	let keys = [
		"glossary/governments/uk_political_parties.json",
		"glossary/governments/us_federal_agencies.json",
		"glossary/humans/us_presidents.json",
		"glossary/geography/countries.json"
	].to_vec();
	let data = get_required_assets(keys);
	let mut ndata: Vec<String> = Vec::new();
	for m in data["glossary/governments/us_federal_agencies.json"]["agencies"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/governments/uk_political_parties.json"]["parties"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/geography/countries.json"]["countries"].members() {
		ndata.push(m.to_string());
	}
	for m in data["glossary/humans/us_presidents.json"]["objects"].members() {
		ndata.push(m["person"]["name"].to_string());
	}
	return ndata;
}

pub fn load_bert_dict() -> json::JsonValue {
	let data = get_required_assets(["glossary/bert-dict.json"].to_vec());
	let d = json::parse(&data["glossary/bert-dict.json"].to_string()).unwrap();
	return d;
}

pub fn get_required_assets(keys:Vec<&str>) -> json::JsonValue {
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