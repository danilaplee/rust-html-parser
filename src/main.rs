extern crate whatlang;
extern crate select;
extern crate futures;

use std::io;
use std::fs::{self, DirEntry};
use std::fs::File;
use std::path::Path;
use std::env;
use std::thread;
use std::io::BufReader;
use std::net::SocketAddr;
use whatlang::{detect, Lang, Script};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use futures::executor::block_on;



fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
            	visit_dirs(&path);
            } else {
			    thread::spawn(move || {
		            parse_file(&entry);
			    });
            }
        }
    }
    Ok(())
}

fn main() {
    println!("Running tgnews v0.1");
    let args: Vec<String> = env::args().collect();

    let query = &args[1];
    let filename = &args[2];

    println!("Searching for {}", query);
    println!("In folder {}", filename);
    let path = Path::new(filename);
    let result = visit_dirs(path);
    println!("ALL DONE!");
}


fn parse_file(entry: &DirEntry) -> Result<(), Box<dyn std::error::Error + 'static>> {

    let path = entry.path();
	println!("parsing File {:?}", path);
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let document = Document::from_read(reader).unwrap();
    let mut h1 : String = "1".to_string();
    for node in document.find( Name("h1") ) {
        h1 = node.text();
    }
    let info = detect(&h1).unwrap();
    let eng = info.lang() == Lang::Eng;
    let rus = info.lang() == Lang::Rus;
    if eng || rus {
    	println!("eng || rus");
    }
    else {
    	println!("unknown language");
    }
    Ok(())

}

