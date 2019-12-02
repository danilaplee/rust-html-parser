extern crate redis;
extern crate regex;
use regex::Regex;
use json::JsonValue;
use json::object;
use json::array;
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
use threadpool::ThreadPool;

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

pub fn start_bigquery_service(
	index:Arc<Mutex<Index>>, 
	db:Arc<Mutex<JsonValue>>, 
	schema:Schema, 
	query:String, 
	names_db:Arc<Mutex<BTreeMap<String, String>>>
) {
    let pool = ThreadPool::with_name("bq_pool1".into(), 1);
    pool.execute(move || {
		let gamesg 				= librarian::load_games_glossary();
		let bert 				= librarian::load_bert_dict();
		let sportsg				= librarian::load_sports_glossary();
		let tvg 				= librarian::load_etv_glossary();
		let corpg 				= librarian::load_corp_glossary();
		let scienceg 			= librarian::load_science_glossary();
		let medicineg 			= librarian::load_medicine_glossary();
		let techg				= librarian::load_tech_glossary();
		let govg 				= librarian::load_gov_glossary();
		let musicg 				= librarian::load_music_glossary();
		let bookg 			 	= librarian::load_book_glossary();
		let artg 			 	= librarian::load_art_glossary();
		let terrorg 			= librarian::load_terror_glossary();
		let opsg 				= librarian::load_ops_glossary();
	    let games 	 			= find_bq_score(index.clone(), schema.clone(), &gamesg, "games");
	    let sports				= find_bq_score(index.clone(), schema.clone(), &sportsg, "sports");
	    let corp				= find_bq_score(index.clone(), schema.clone(), &corpg, "corp");
	    let science				= find_bq_score(index.clone(), schema.clone(), &scienceg, "science");
	    let medicine			= find_bq_score(index.clone(), schema.clone(), &medicineg, "medicine");
	    let tech				= find_bq_score(index.clone(), schema.clone(), &techg, "tech");
	    let gov 				= find_bq_score(index.clone(), schema.clone(), &govg, "gov");
	    let music 				= find_bq_score(index.clone(), schema.clone(), &musicg, "music");
	    let book 				= find_bq_score(index.clone(), schema.clone(), &bookg, "book");
	    let terror 				= find_bq_score(index.clone(), schema.clone(), &terrorg, "terror");
	    let tv 					= find_bq_score(index.clone(), schema.clone(), &tvg, "tv");
	    let art 				= find_bq_score(index.clone(), schema.clone(), &artg, "art");
	    let mut json 			= object!{};
	    let mut news 			= object!{"articles"=>json::JsonValue::new_array()};
	    let mut categories 		= json::JsonValue::new_array();
	    let mut ncategories:BTreeMap<String, String> 	= BTreeMap::new();
	    let ctypes 				= ["society", "economy", "technology", "sports", "entertainment", "science", "other"].to_vec();
	    let mut ctypestree:BTreeMap<String, usize> = BTreeMap::new();
    	let mut top = array![
	    	object!{
	    		"category"=> "any",
	    		"threads" => json::JsonValue::new_array()
	    	}
    	];
	    for (i, ct) in ctypes.iter().enumerate() {
	    	let ct2 = ct.to_string();
	    	let ct3 = ct.to_string();
		    categories.push(object!{
		    	"category" => ct.to_string(),
		    	"articles" => array!{}
		    });
		    top.push(object!{
	    		"category" => ct2,
	    		"threads"  => json::JsonValue::new_array()
	    	});
	    	ctypestree.insert(ct3, i+1);
	    }
	    
	    for (key, items) in bert.entries() {
	    	let schema_clone = schema.clone();
	    	let mut itemsref:Vec<String> = [].to_vec();
	    	for item in items.members() {
	    		itemsref.push(item.to_string());
	    	}
	    	let score_docs = find_bq_score(index.clone(), schema_clone, &itemsref, &key);
	    	for (ikey, score_doc) in score_docs.entries() {
	    		if json[ikey].is_null() {
	    			json[ikey] = json::parse(&score_doc.dump()).unwrap();
	    			json[ikey][key] = score_doc["score"].to_string().into();
	    			json[ikey].remove("score");
	    		}
	    		else {
	    			json[ikey][format!("word_{}", key)] = score_doc[format!("word_{}", key)].to_string().into();
	    			json[ikey][key] = score_doc["score"].to_string().into();
	    		}
	    	}
	    }
	    // println!("listing json items: {:?}", &json.entries().count());
	    for (path, item) in json.entries() {
	    	let org = !(item["org"].is_null());
	    	let gpe = !(item["gpe"].is_null());
	    	let person = !(item["person"].is_null());
	    	let money = !(item["money"].is_null());
	    	let art = !(item["art"].is_null());
	    	let product = !(item["product"].is_null());
	    	let igames = !(games[path].is_null());
	    	let igov  = !(gov[path].is_null());
	    	let isports  = !(sports[path].is_null());
	    	let itech  = !(tech[path].is_null());
	    	let icorp  = !(corp[path].is_null());
	    	let imusic  = !(music[path].is_null());
	    	let ibook  = !(book[path].is_null());
	    	let iscience  = !(science[path].is_null());
	    	let imedicine  = !(medicine[path].is_null());
	    	let itv  = !(tv[path].is_null());
	    	let mut is_news:bool = false;
			let re2 = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
	    	let title2 = re2.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
	    	// println!("found item: {:?}, org: {}, gpe: {}, person: {}, money: {}, itech: {}, icorp: {}, isports: {}, iproduct: {}", &title2, &org, &gpe, &person, &money, &itech, &icorp, &isports, &product);
			
			//SOCIETY
			if org && (gpe || igov || icorp || money || art || person) {
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		    	let title = re.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
				is_news = true;
				categories[0]["articles"].push(path);
				ncategories.insert(title, "society".to_string());
			}

			//ECONOMY
			if !is_news && money {
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		    	let title = re.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
				is_news = true;
				categories[1]["articles"].push(path);
				ncategories.insert(title, "economy".to_string());
			}

			//TECH
			if !is_news && (itech || product) {
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		    	let title = re.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
				is_news = true;
				categories[2]["articles"].push(path);
				ncategories.insert(title, "technology".to_string());
			}

			//SPORTS
			if !is_news && isports {
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		    	let title = re.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
				is_news = true;
				categories[3]["articles"].push(path);
				ncategories.insert(title, "sports".to_string());
			}

			//ENTERTAINMENT
			if !is_news && (art || igames || imusic || ibook || itv)  {
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		    	let title = re.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
				is_news = true;
				categories[4]["articles"].push(path);
				ncategories.insert(title, "entertainment".to_string());
			}

			//SCIENCE
			if !is_news && (iscience || imedicine) {
				let re = Regex::new(r"/[^A-Za-z0-9 ]/").unwrap();
		    	let title = re.replace_all(&item["title"][0].to_string().to_lowercase(), "").to_string();
				is_news = true;
				categories[5]["articles"].push(path);
				ncategories.insert(title, "science".to_string());
			}

			//NEWS
			if is_news == true {
				news["articles"].push(path);
			}
			else {
				if org || gpe || person || igov || itech || icorp {
					categories[6]["articles"].push(path);
				}
			}

	    }
	    // println!("BTreeMap ncategories {:?}", ncategories);
	    if query == "news" {
	    	println!("{}", news.pretty(2));
	    }

	    if query == "categories" {
	    	println!("{}", categories.pretty(2));
	    }

	    if query == "threads"
	    || query == "top"
	    || query == "debug" {
		    let self_occurences = find_self_occurences(index.clone(), schema.clone(), names_db);
	    	let mut thr = json::JsonValue::new_array();
		    for (title, items) in self_occurences.entries() {
				let re = Regex::new(r"/[^A-Za-z0-9]/").unwrap();
		    	let title2 = re.replace_all(&String::from(title).to_lowercase(), "").to_string();
		    	if items.len() < 2 {
		    		continue;
		    	}
		    	else {
		    		let mut articles = object!{
				    	"title" => title,
				    	"articles" => json::JsonValue::new_array()
				    };
				    for i in items.members() {
				    	let ii = i.to_string();
				    	articles["articles"].push(ii);
				    }
				    if query == "top" {
				    	let other = &"other".to_string();
				    	let mut category = ncategories.get(&title2);
				    	if category == None {
				    		category = Some(other);
				    	}
				    	let catindex = (ctypes.iter().position(|&r| r == category.unwrap().to_string()).unwrap() + 1);
				    	articles["category"] = json::JsonValue::String(category.unwrap().to_string());
				    	let articles2 = json::parse(&articles.dump()).unwrap();
				    	let articles3 = json::parse(&articles.dump()).unwrap();

				    	top[0]["threads"].push(articles2);
				    	top[catindex]["threads"].push(articles3);
				    }
			    	thr.push(articles);
		    	}
		    }
		    if query == "threads" {
		    	let threads_result = sort_by_thread_count(&thr);
	    		println!("{}", threads_result.pretty(2));
		    }
		    if query == "top" {
		    	let mut current = 0;
		    	let mut ntop = array![];
			    for gthreads in top.members() {
			    	ntop[current] = object!{
			    		"category" => gthreads["category"].to_string(),
			    		"threads" => sort_by_thread_count(&gthreads["threads"])
			    	};
			    	current += 1;
			    }
	    		println!("{}", ntop.pretty(2));
		    }
	    }



	});
	pool.join();
}
fn sort_by_thread_count(thr:&JsonValue) -> JsonValue {
	let mut threads_result:JsonValue = json::JsonValue::new_array();
    let mut vecthr:Vec<&JsonValue> = thr.members().collect();
    vecthr.sort_by(|a,b| {
    	let ai:usize = a["articles"].members().count();
    	let bi:usize = b["articles"].members().count();
    	bi.cmp(&ai)
    });

    for v in &vecthr {
    	let vv = json::parse(&v.pretty(1)).unwrap();
    	threads_result.push(vv);
    }
    return threads_result;
}

fn find_bq_score(_index:Arc<Mutex<Index>>, schema:Schema, theme:&Vec<String>, tname:&str) -> JsonValue {

	let args: Vec<String> = env::args().collect();
	let query	 = &args[1];
	let mut _score = 0;
    let mut j 	= object!{};
    let mut lock_sucess = false;
    let mut lock = _index.try_lock();
    while !lock_sucess {
	    if let Ok(ref index) = lock {
	    	let reader = index.reader().unwrap();
			let searcher = reader.searcher();

	        let title = schema.get_field("title").unwrap();
		    let body = schema.get_field("body").unwrap();

			let query_parser = QueryParser::for_index(&index, vec![title, body]);
			
			for word in theme {
				let re = Regex::new(r"/[^A-Za-z0-9]/").unwrap();
				let q = re.replace_all(word.as_str(), "").to_string();
				let query = query_parser.parse_query(&q);
			    match query {
			    	Ok(query) => {
						let top_docs: Vec<(Score, DocAddress)> =
					    searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

						for (sc, doc_address) in top_docs {
							let mut min_score = 15;
							if &tname != &"games" {
								min_score = 8;
							}
							if &tname == &"science" {
								min_score = 10;
							}
							if sc >= min_score as f32 {
							    let retrieved_doc = searcher.doc(doc_address);
							    match retrieved_doc {
							    	Ok(ref doc) => {
							    		let mut js = json::parse(&schema.to_json(&doc)).unwrap();
							    		let keystr = js["body"][0].to_string();
							    		js["score"] = sc.into();
							    		js[format!("word_{}", tname)] = word.to_string().into();
							    		j[keystr] = js;
							    	},
							    	Err(err) => {
							    		continue;
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

fn find_self_occurences(_index:Arc<Mutex<Index>>, schema:Schema, names_db:Arc<Mutex<BTreeMap<String, String>>>) -> JsonValue {
	let args: Vec<String> = env::args().collect();
	let query	 = &args[1];
	let mut _score = 0;
    let mut j 	= object!{};
    let mut lock_sucess = false;
    let mut lock = _index.try_lock();
    let mut reverse_map = BTreeMap::new();
    while !lock_sucess {
	    if let Ok(ref index) = lock {
	    	let reader = index.reader().unwrap();
			let searcher = reader.searcher();

	        let title = schema.get_field("title").unwrap();
		    let body = schema.get_field("body").unwrap();

			let query_parser = QueryParser::for_index(&index, vec![title, body]);

		    let mut lock2 = names_db.try_lock();
		    if let Ok(ref names) = lock2 {
		    	let n:&BTreeMap<String, String> = names; 
				for (word, _) in n {
					if reverse_map.contains_key(word) {
						continue;
					}
					let q:&str = word.as_str();
					let query = query_parser.parse_query(q);
				    match query {
				    	Ok(query) => {
							let top_docs: Vec<(Score, DocAddress)> =
						    searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
							for (sc, doc_address) in top_docs {
								let mut min_score = 15;
								if sc >= min_score as f32 {
								    let retrieved_doc = searcher.doc(doc_address);
								    match retrieved_doc {
								    	Ok(ref doc) => {
								    		let mut js = json::parse(&schema.to_json(&doc)).unwrap();
										    reverse_map.insert(js["title"][0].to_string(), js["body"][0].to_string());
								    		let keystr = js["body"][0].to_string();
								    		js["score"] = sc.into();
								    		js["occurence"] = word.to_string().into();
								    		if j[q].is_null() {
								    			j[q] = json::JsonValue::new_array();
								    		}
								    		j[q].push(keystr);
								    	},
								    	Err(err) => {
								    		continue;
								    	}
								    }
								}
					    	}
					    },
				    	Err(err) => {
					    	continue;
				    	}
					}
				}
				lock_sucess = true;
			}
			drop(&lock);
		}
		else {
			thread::sleep(time::Duration::from_nanos(10000));
			drop(&lock);
		}
    }
	return j;
}

