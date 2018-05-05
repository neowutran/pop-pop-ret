extern crate docopt;
extern crate glob;
extern crate goblin;
extern crate num_cpus;
extern crate regex;
extern crate threadpool;
#[macro_use]
extern crate serde_derive;
use docopt::Docopt;
use glob::glob;
use regex::Regex;
use std::{str, collections::HashSet, fs::File, io::prelude::*, sync::mpsc,
          sync::mpsc::{Receiver, Sender}};
use threadpool::ThreadPool;
const USAGE: &'static str = "
Find hexadecimal string inside a file.

Usage:
  pop_pop_ret <files>... [--regex <regex>] [--bad-bytes <bad_bytes> | --good-bytes <good_bytes>] [--aslr]
  pop_pop_ret (-h | --help)

Options:
  -h --help                                     Show this screen.
  --regex <regex>, -r <regex>                   Execute regex. [default: (07|17|1F|58|59|5A|5B|5C|5D|5E|5F){2}(C2|C3|CB|CA)]
  --bad-bytes <bad_bytes>, -b <bad_bytes>       List of forbidden bytes. [default: ]
  --good-bytes <good_bytes>, -g <good_bytes>    List of allowed bytes. [default: ]
  --aslr                                        No bad/good char check on the image_base + offset adresse.
";

#[derive(Deserialize)]
struct Args {
    arg_files: Vec<String>,
    flag_regex: String,
    flag_bad_bytes: String,
    flag_good_bytes: String,
    flag_aslr: bool,
}

fn parse_bytes(arg: &str) -> HashSet<u8> {
    let mut result = HashSet::new();
    for byte in arg.split("\\x") {
        if byte.is_empty() {
            continue;
        }
        let value = u8::from_str_radix(&byte, 16).expect(&format!(
            "Not an hexadecimal string: {}. Expect something like 'FF'",
            &byte
        ));
        result.insert(value);
    }
    result
}

fn byte_allowed(byte: u8, bad_bytes: &HashSet<u8>, good_bytes: &HashSet<u8>) -> bool {
    if bad_bytes.contains(&byte) || (!good_bytes.contains(&byte) && !good_bytes.is_empty()) {
        return false;
    }
    true
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let bad_bytes = parse_bytes(&args.flag_bad_bytes);
    let good_bytes = parse_bytes(&args.flag_good_bytes);
    let regex = Regex::new(&args.flag_regex).unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let full_cpus = num_cpus::get();
    let mut usable_cpus = full_cpus - 1;
    println!("Number of virtual core: {}", full_cpus);
    if usable_cpus <= 1 {
        usable_cpus = 1;
    }
    let thread_pool_decompress: ThreadPool = ThreadPool::new(usable_cpus);
    for file in &args.arg_files {
        for entry in glob(&file).expect("Failed to read glob pattern") {
            let os_string = entry.unwrap().into_os_string();
            let string = os_string.into_string().unwrap();
            let thread_tx = tx.clone();
            let regex_clone = regex.clone();
            let bad_bytes_clone = bad_bytes.clone();
            let good_bytes_clone = good_bytes.clone();
            let aslr = args.flag_aslr.clone();
            thread_pool_decompress.execute(move || {
                let mut hex_array = Vec::new();
                let mut image_base = 0;
                {
                    let mut binary = Vec::new();
                    {
                        let mut file = File::open(string.clone()).expect("file not found");
                        file.read_to_end(&mut binary).unwrap();
                        let res = goblin::Object::parse(&binary).unwrap();
                        println!("{:#?}", res);
                        match res {
                            goblin::Object::PE(pe) => match pe.header.optional_header {
                                Some(header) => {
                                    image_base = header.windows_fields.image_base as usize;
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                    for element in binary.iter() {
                        hex_array.push(format!("{:02X}", element));
                    }
                }
                for mat in regex_clone.find_iter(&hex_array.join("")) {
                    let offset_raw = mat.start() / 2;
                    let offset_and_image_base = offset_raw + image_base;

                    let byte1_1 = (offset_raw & 0x000000FF) as u8;
                    let byte2_1 = ((offset_raw & 0x0000FF00) >> 8) as u8;
                    let byte1_2 = (offset_and_image_base & 0x000000FF) as u8;
                    let byte2_2 = ((offset_and_image_base & 0x0000FF00) >> 8) as u8;
                    let byte3_2 = ((offset_and_image_base & 0x00FF0000) >> 16) as u8;
                    let byte4_2 = ((offset_and_image_base & 0xFF000000) >> 24) as u8;
                    if !(byte_allowed(byte1_1, &bad_bytes_clone, &good_bytes_clone)
                        && byte_allowed(byte2_1, &bad_bytes_clone, &good_bytes_clone))
                    {
                        continue;
                    }

                    if !aslr
                        && (!(byte_allowed(byte1_2, &bad_bytes_clone, &good_bytes_clone)
                            && byte_allowed(byte2_2, &bad_bytes_clone, &good_bytes_clone)
                            && byte_allowed(byte3_2, &bad_bytes_clone, &good_bytes_clone)
                            && byte_allowed(byte4_2, &bad_bytes_clone, &good_bytes_clone)))
                    {
                        continue;
                    }

                    let contents =
                        format!("{}:{:x}:{:x}", string, offset_raw, offset_and_image_base);
                    thread_tx.send(contents).unwrap();
                }
            });
        }
    }

    drop(tx);
    for received in rx {
        println!("{}", received);
    }
}
