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
use std::{
    collections::HashSet, fs::File, io::prelude::*, str, sync::mpsc, sync::mpsc::{Receiver, Sender},
};
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
  --aslr, -a                                    No bad/good char check on the image_base + offset adresse.

Example:
pop_pop_ret ./*.dll -g '\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0b\x0c\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2a\x2b\x2c\x2d\x2e\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3b\x3c\x3d\x3e\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f'
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
        result.insert(u8::from_str_radix(&byte, 16).expect(&format!(
            "Not an hexadecimal string: {}. Expect something like 'FF'",
            &byte
        )));
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
                let mut sections = Vec::new();
                {
                    let mut binary = Vec::new();
                    {
                        let mut file = File::open(string.clone()).expect("file not found");
                        file.read_to_end(&mut binary).unwrap();
                        let res = goblin::Object::parse(&binary).unwrap();
                        //println!("{:#?}", res);
                        match res {
                            goblin::Object::PE(pe) => {
                                match pe.header.optional_header {
                                    Some(header) => {
                                        image_base = header.windows_fields.image_base as usize;
                                    }
                                    _ => (),
                                }
                                sections = pe.sections;
                            }
                            _ => (),
                        }
                    }
                    for element in binary.iter() {
                        hex_array.push(format!("{:02X}", element));
                    }
                }
                'found: for mat in regex_clone.find_iter(&hex_array.join("")) {
                    let mut offset_raw = (mat.start() / 2) as u32;
                    let mut offset_and_image_base = offset_raw + image_base as u32;
                    let mut delta: Option<u32> = None;
                    for section in &sections {
                        if offset_raw >= section.pointer_to_raw_data
                            && offset_raw <= section.pointer_to_raw_data + section.size_of_raw_data
                        {
                            delta = Some(section.virtual_address - section.pointer_to_raw_data);
                            break;
                        }
                    }
                    match delta {
                        Some(value) => {
                            offset_raw += value;
                            offset_and_image_base += value;
                        }
                        None => panic!("unable to find delta value"),
                    }

                    let byte1_1 = (offset_raw & 0xFF) as u8;
                    let byte2_1 = ((offset_raw & (0xFF << 8)) >> 8) as u8;
                    if !(byte_allowed(byte1_1, &bad_bytes_clone, &good_bytes_clone)
                        && byte_allowed(byte2_1, &bad_bytes_clone, &good_bytes_clone))
                    {
                        continue;
                    }

                    let mut offset_and_image_base_bytes = Vec::new();
                    offset_and_image_base_bytes.push((offset_and_image_base & 0xFF) as u8);
                    offset_and_image_base_bytes
                        .push(((offset_and_image_base & (0xFF << 8)) >> 8) as u8);
                    offset_and_image_base_bytes
                        .push(((offset_and_image_base & (0xFF << 16)) >> 16) as u8);
                    offset_and_image_base_bytes
                        .push(((offset_and_image_base & (0xFF << 24)) >> 24) as u8);
                    if !aslr {
                        for byte in offset_and_image_base_bytes {
                            if !byte_allowed(byte, &bad_bytes_clone, &good_bytes_clone) {
                                continue 'found;
                            }
                        }
                    }
                    thread_tx
                        .send(format!(
                            "{}\t{:x}\t{:x}",
                            string, offset_raw, offset_and_image_base
                        ))
                        .unwrap();
                }
            });
        }
    }
    drop(tx);
    for received in rx {
        println!("{}", received);
    }
}
