extern crate redis;
use redis::Commands;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub fn print_languages_start() {

		let lang_en = r#"[
	{
		"lang_code": "en",
		"articles": ["#;

		println!("{}", lang_en);
}

pub fn print_news_start() {

		let start = r#"{
		"articles": ["#;

		println!("{}", start);
}

pub fn print_news_end() {

		let end = r#"		]
}"#;

		println!("{}", end);
}
pub fn print_languages_end(ruDB:Arc<Mutex<Vec<String>>>) {

		let mut rus_data:Vec<String> = Vec::new();
	    let mut lock = ruDB.try_lock();
	    if let Ok(ref mut mtx) = lock {
	       	rus_data = mtx.to_vec();
	    } else {
	        // println!("languages end try_lock failed");
	    }
	    drop(lock);

		let lang_ru = r#"		]
	},
	{
		"lang_code": "ru",
		"articles": ["#;
		
		let lang_end = r#"		]
	}
]"#;
		println!("{}", lang_ru);
		for item in rus_data {
			println!(r#"		{:?}, "#, item);
		}
		println!("{}", lang_end);
}