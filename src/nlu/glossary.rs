extern crate redis;
use redis::Commands;
use redis::RedisResult;
use std::collections::HashSet;
use json::object;
use json::JsonValue;
use std::fs::File;
use std::path::Path;
use std::env;
use std::thread;
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

pub fn start_glossary() {

	// let sports:Vec<&str> = load_sports_glossary();
	// let games:Vec<&str> = load_games_glossary();
}

pub fn process_word() {

}

fn load_sports_glossary()  {
	let data = load_glossary("sports");
}

fn load_games_glossary() {
	let data = load_glossary("games");
}

fn load_glossary(glossary_type:&str) {

}