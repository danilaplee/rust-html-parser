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
use std::collections::BTreeMap;
use tantivy::{Index, IndexReader,ReloadPolicy, DocAddress, Score, doc};
use tantivy::schema::*;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

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
	// test_fuzzy();
    thread::spawn(move || {
		// let games:Vec<String> 	= librarian::load_games_glossary();
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
				// +&games.len()
			);
		}
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
			    		&ops
			    	);
			    	if(scores["highest_value"].as_u64().unwrap() > 0) {
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
		    } else {
			    drop(lock);
		    }
		    let _millis = time::Duration::from_nanos(offset);
			thread::sleep(_millis);
		}
    });
}

pub fn start_bigquery_service(index:Arc<Mutex<Index>>, db:Arc<Mutex<JsonValue>>, schema:Schema) {
    thread::spawn(move || {
		let games:Vec<String> 	= librarian::load_games_glossary();
	    let args: Vec<String> 	= env::args().collect();
	    let query	 			= &args[1];
	    let games_scores 		= find_bq_score(index, schema, &games, "games");
	}).join();
}

fn find_bq_score(_index:Arc<Mutex<Index>>, schema:Schema, theme:&Vec<String>, tname:&str) -> JsonValue {

	let args: Vec<String> = env::args().collect();
	let query	 = &args[1];
	let mut _score = 0;
    let mut j 	= object!{};
    let mut lock_sucess = false;
    // println!("========== start lock =========");
    let mut lock = _index.try_lock();
    while !lock_sucess {
	    if let Ok(ref index) = lock {
	    	// println!("======== lock success ========");
	    	let reader = index.reader().unwrap();
	    	// println!("======== got reader success ========");
			let searcher = reader.searcher();

	        let title = schema.get_field("title").unwrap();
		    let body = schema.get_field("body").unwrap();
	    	// println!("======== got fields success ========");

			let query_parser = QueryParser::for_index(&index, vec![title, body]);

			
			for word in theme {
				let q:&str = word.as_str();
				let query = query_parser.parse_query(q);
			    match query {
			    	Ok(query) => {
						let top_docs: Vec<(Score, DocAddress)> =
					    searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

						for (sc, doc_address) in top_docs {
							if sc >= q.len() as f32 {
							    let retrieved_doc = searcher.doc(doc_address);
							    match retrieved_doc {
							    	Ok(ref doc) => {
									    println!("Found key: {:?} score: {}  doc: {:?}",q, sc, schema.to_json(&doc));
							    	},
							    	Err(err) => {
								    	println!("Found Game News with  but error: {:?}", &err);

							    	}
							    }
							}
				    	}
				    },
			    	Err(err) => {
				    	// println!("query parsing error: {:?}", &err);
				    	continue;
			    	}
				}
			}
			lock_sucess = true;
			drop(&lock);
		}
		else {
			thread::sleep(time::Duration::from_nanos(10000));
			drop(&lock);
		}
    }
	return j;
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
	ops:&Vec<String>
) -> json::JsonValue {
    let args:Vec<String>= env::args().collect();
    let query	 		= &args[1];
	
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
	let scores = object!{
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
		// "ops_score" => ops_score,
	};
	let mut highest_key = "";
	let mut highest_value = 0;
	for score in scores.entries() {
		let (key, value) = score;
		let v = value.as_u64().unwrap();
		if v > highest_value {
			highest_value = v;
			highest_key = key;
		}
	}
	if highest_value > 0 {
		if query == "debug" {
			// println!("news worthy: {}, {:?}",&highest_key, &h1);
			// println!("scores: {}", &scores.pretty(2));
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
		"highest_value" => highest_value,
		"highest_key" => highest_key
	};
}

fn find_theme_score(item:&JsonValue,theme:&Vec<String>, tname:&str) -> i64 {

	let args: Vec<String> = env::args().collect();
	let query	 = &args[1];
	let mut _score = 0;
	let h1 = &item["h1"].to_string();
	// let cc:Vec<&str> = Vec::new()
	for c in theme {

		if tname == "games" {
			if h1.to_lowercase().contains(c)  {
				_score += 1;
				if query == "debug" {
					// println!("found game: {:?}", c);
				}
			}
		} else {
			let sc = fuzzy_indices(&h1, &c);
			if sc != None {
				let (score, indices) = &sc.unwrap();
				if score > &90 {
					_score += 1;
					if query == "debug" {
						if tname == "sports" {
							// println!("found sports: {:?} in {}", &c, &h1);
						}
					}
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
