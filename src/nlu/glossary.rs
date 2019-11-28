extern crate redis;
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

use packer::Packer;

#[derive(Packer)]
#[packer(source = "glossary")]
struct Assets;

use super::librarian;


// Society (includes Politics, Elections, Legislation, Incidents, Crime)
// Economy (includes Markets, Finance, Business)
// Technology (includes Gadgets, Auto, Apps, Internet services)
// Sports (includes E-Sports)
// Entertainment (includes Movies, Music, Games, Books, Arts)
// Science (includes Health, Biology, Physics, Genetics)
// Other (news articles that don't fall into any of the above categories)

pub fn start(
	done_index:Arc<Mutex<Vec<String>>>, 
	queue:Arc<Mutex<VecDeque<JsonValue>>>, 
	db:Arc<Mutex<JsonValue>>,
	offset:u64
) {
    thread::spawn(move || {
	    let args: Vec<String> = env::args().collect();
	    let query	 = &args[1];

		let sports:Vec<String> 	= librarian::load_sports_glossary();
		let etv:Vec<String> 	= librarian::load_etv_glossary();
		let corp:Vec<String> 	= librarian::load_corp_glossary();
		let science:Vec<String> = librarian::load_science_glossary();
		let medicine:Vec<String>= librarian::load_medicine_glossary();
		let tech:Vec<String>	= librarian::load_tech_glossary();
		let gov:Vec<String> 	= librarian::load_gov_glossary();
		let music:Vec<String> 	= librarian::load_music_glossary();
		let book:Vec<String> 	= librarian::load_book_glossary();
		let art:Vec<String> 	= librarian::load_art_glossary();
		let terror:Vec<String> 	= librarian::load_terror_glossary();
		let ops:Vec<String> 	= librarian::load_ops_glossary();
		let games:Vec<String> 	= librarian::load_games_glossary();
		if query == "debug" {
			println!("library loaded: {:?}", 
				&corp.len()
				+&etv.len()
				+&sports.len()
				+&science.len()
				+&medicine.len()
				+&tech.len()
				+&gov.len()
				+&music.len()
				+&book.len()
				+&art.len()
				+&terror.len()
				+&ops.len()
				+&games.len()
			);
		}
		// println!("sports glossary length: {:?}", sports.len());
		// println!("etv glossary length: {:?}", etv.len());
		// println!("corp glossary length: {:?}", corp.len());
		// println!("science glossary length: {:?}", science.len());
		// println!("medicine glossary length: {:?}", medicine.len());
		// println!("tech glossary length: {:?}", tech.len());
		// println!("gov glossary length: {:?}", gov.len());
		// println!("music glossary length: {:?}", music.len());
		// println!("book glossary length: {:?}", book.len());
		// println!("art glossary length: {:?}", art.len());
		// println!("ops glossary length: {:?}", ops.len());
		// println!("terror glossary length: {:?}", terror.len());
		let mut not_finished = true;
		while not_finished {
		    let mut lock = queue.try_lock();
		    if let Ok(ref mut mtx) = lock {
		    	let mut item = mtx.pop_front();
			    drop(lock);
		    	if item != None {
			    	let mut nitem = item.unwrap();
			    	let scores = process_item(
			    		&nitem, 
			    		&sports, 
			    		&corp, 
			    		&medicine, 
			    		&science, 
			    		&tech,
			    		&etv,
			    		&gov,
			    		&music,
			    		&art,
			    		&book,
			    		&terror,
			    		&ops,
			    		&games
			    	);
				    let mut lock2 = db.try_lock();
				    if let Ok(ref mut mtx2) = lock2 {
				    	// println!("mtx 2: {:?}", mtx2);
				    	let id = &nitem["path"].to_string();
				    	if mtx2[id].is_null() {
					    	nitem["scores"] = scores;
					    	mtx2[id] = nitem;
				    	}
				    	else {
					    	mtx2[id]["scores"] = scores;
				    	}
				    }
			    	drop(lock2);
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
	corp:&Vec<String>, 
	medicine:&Vec<String>,
	science:&Vec<String>,
	tech:&Vec<String>,
	etv:&Vec<String>,
	gov:&Vec<String>,
	music:&Vec<String>,
	art:&Vec<String>,
	book:&Vec<String>,
	terror:&Vec<String>,
	ops:&Vec<String>,
	games:&Vec<String>
) -> json::JsonValue {
    let args: Vec<String> = env::args().collect();
    let query	 = &args[1];
	
	let h1 				= &item["h1"].to_string();
	let corp_score 		= find_theme_score(&item, corp, "corp");
	let sports_score 	= find_theme_score(&item, sports, "sports");
	let medicine_score 	= find_theme_score(&item, medicine, "medicine");
	let science_score 	= find_theme_score(&item, science, "science");
	let tech_score 		= find_theme_score(&item, tech, "tech");
	let etv_score 		= find_theme_score(&item, etv, "etv");
	let gov_score 		= find_theme_score(&item, gov, "gov");
	let music_score 	= find_theme_score(&item, music, "music");
	let art_score 		= find_theme_score(&item, art, "art");
	let book_score 		= find_theme_score(&item, book, "book");
	let terror_score 	= find_theme_score(&item, terror, "terror");
	let ops_score 		= find_theme_score(&item, ops, "ops");
	let games_score 	= find_theme_score(&item, games, "games");
	if corp_score > 80 || sports_score > 80 || medicine_score > 80
	|| science_score > 80 || tech_score > 80 || etv_score > 80 || gov_score > 80
	|| music_score > 80 || art_score > 80 || book_score > 200 
	|| ops_score > 500 || terror_score > 100 ||  games_score > 80 {
		if query == "debug" {
			println!("news worthy: {:?}", &h1);
			println!("corp score {}", &corp_score);
			println!("sports score {}", &sports_score);
			println!("medicine score {}", &medicine_score);
			println!("science score {}", &science_score);
			println!("tech score {}", &tech_score);
			println!("etv score {}", &etv_score);
			println!("gov score {}", &gov_score);
			println!("book score {}", &book_score);
			println!("music score {}", &music_score);
			println!("art score {}", &art_score);
			println!("ops score {}", &ops_score);
			println!("terror score {}", &terror_score);
			println!("games score {}", &games_score);
		}
	}
	return object!{
		"corp_score" => corp_score,
		"sports_score" => sports_score,
		"medicine_score" => medicine_score,
		"science_score" => science_score,
		"tech_score" => tech_score,
		"etv_score" => etv_score,
		"gov_score" => gov_score,
		"music_score" => music_score,
		"art_score" => art_score,
		"book_score" => book_score,
		"terror_score" => terror_score,
		"ops_score" => ops_score,
		"games_score" => games_score
	};
	// return item;
}

fn find_theme_score(item:&JsonValue,theme:&Vec<String>, tname:&str) -> i64 {

	let mut _score = 0;
	let h1 = &item["h1"].to_string();
	for c in theme {
		let sc = fuzzy_indices(&h1, &c);
		if sc != None {
			let (score, indices) = sc.unwrap();
			if tname == "games" {
				 score > c.len() as i64 {
					_score += score;
				}
			} else {

				if score > (c.len()/2) as i64 {
					_score += score;
				}
			}
		}
	}
	return _score;
}

pub fn process_text(text: &str) {
	let (score, indices) = fuzzy_indices("axbycz", "abc").unwrap();
	assert_eq!(indices, [0, 2, 4]);
}
