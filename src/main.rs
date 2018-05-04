extern crate docopt;
extern crate glob;
extern crate num_cpus;
extern crate regex;
extern crate threadpool;
#[macro_use]
extern crate serde_derive;
use docopt::Docopt;
use glob::glob;
use regex::Regex;
use std::{str, fs::File, io::prelude::*, sync::mpsc, sync::mpsc::{Receiver, Sender}};
use threadpool::ThreadPool;
const USAGE: &'static str = "
Find hexadecimal string inside a file.

Usage:
  pop_pop_ret <file> [--regex <regex>]
  pop_pop_ret (-h | --help)

Options:
  -h --help                         Show this screen.
  --regex <regex>                   Execute regex. [default: (07|17|1F|58|59|5A|5B|5C|5D|5E|5F){2}(C2|C3|CB|CA)]
";

#[derive(Deserialize)]
struct Args {
    arg_file: String,
    flag_regex: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let regex = Regex::new(&args.flag_regex).unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let full_cpus = num_cpus::get();
    let mut usable_cpus = full_cpus - 1;
    println!("Number of virtual core: {}", full_cpus);
    if usable_cpus <= 1 {
        usable_cpus = 1;
    }
    let thread_pool_decompress: ThreadPool = ThreadPool::new(usable_cpus);
    for entry in glob(&args.arg_file).expect("Failed to read glob pattern") {
        let os_string = entry.unwrap().into_os_string();
        let string = os_string.into_string().unwrap();
        let thread_tx = tx.clone();
        let regex_clone = regex.clone();
        thread_pool_decompress.execute(move || {
            let mut hex_array = Vec::new();
            {
                let mut binary = Vec::new();
                {
                    let mut file = File::open(string.clone()).expect("file not found");
                    file.read_to_end(&mut binary).unwrap();
                }
                for element in binary.iter() {
                    hex_array.push(format!("{:02X}", element));
                }
            }
            for mat in regex_clone.find_iter(&hex_array.join("")) {
                let contents = format!("{}:{:x}", string, mat.start() / 2);
                thread_tx.send(contents).unwrap();
            }
        });
    }

    drop(tx);
    for received in rx {
        println!("{}", received);
    }
}
