extern crate redis;
use redis::Commands;
use redis::RedisResult;
use std::collections::HashSet;

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
pub fn print_languages_end(con: &mut redis::Connection) {

		let rus_data : HashSet<String> = con.smembers("rus".to_string()).unwrap();

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