use clap::{crate_authors, crate_version, AppSettings, Parser};
use glob::glob;
use goblin::pe::section_table::SectionTable;
use rayon::prelude::*;
use regex::bytes::Regex;
use std::{
    collections::HashSet,
    fs::File,
    io::prelude::*,
    str,
    sync::mpsc,
    sync::mpsc::{Receiver, Sender},
};
//#[clap(setting = AppSettings::ColoredHelp)]
#[derive(Parser)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Args {
    #[clap(required = true)]
    files: Vec<String>,
    #[clap(
        short,
        long,
        default_value = r"(\x07|\x17|\x1F|\x58|\x59|\x5A|\x5B|\x5C|\x5D|\x5E|\x5F){2}(\xC2|\xC3|\xCB|\xCA)"
    )]
    regex: String,
    #[clap(short, long)]
    bad_bytes: Option<String>,
    #[clap(short, long)]
    good_bytes: Option<String>,
}

fn parse_bytes(arg: &str) -> HashSet<u8> {
    let mut result = HashSet::new();
    for byte in arg.split("\\x") {
        if byte.is_empty() {
            continue;
        }
        result.insert(u8::from_str_radix(byte, 16).unwrap_or_else(|_| {
            panic!(
                "Not an hexadecimal string: {}. Expect something like 'FF'",
                &byte
            )
        }));
    }
    result
}

fn is_acceptable(
    raw: usize,
    is_64: bool,
    bad_bytes: &HashSet<u8>,
    good_bytes: &HashSet<u8>,
) -> Option<String> {
    let raw_bytes;
    if is_64 {
        raw_bytes = (raw as u64).to_le_bytes().to_vec();
    } else {
        raw_bytes = (raw as u32).to_le_bytes().to_vec();
    }
    for byte in &raw_bytes {
        if bad_bytes.contains(&byte) || (!good_bytes.contains(&byte) && !good_bytes.is_empty()) {
            return None;
        }
    }
    Some(format!("{:08x}", raw))
}

fn verify_search_result(
    search_result: regex::bytes::Match,
    image_base: usize,
    sections: &Vec<SectionTable>,
    bad_bytes: &HashSet<u8>,
    good_bytes: &HashSet<u8>,
    sender: &mut Sender<String>,
    filename: &str,
    is_64: bool,
) {
    let mut offset_raw = search_result.start();
    let mut offset_and_image_base = offset_raw + image_base;
    for section in sections {
        if offset_raw >= section.pointer_to_raw_data as usize
            && offset_raw
                <= (section.pointer_to_raw_data as usize + section.size_of_raw_data as usize)
        {
            let delta = section.virtual_address - section.pointer_to_raw_data;
            offset_raw += delta as usize;
            offset_and_image_base += delta as usize;

            let acceptable_offset = is_acceptable(offset_raw, is_64, bad_bytes, good_bytes);
            let acceptable_offset_and_image_base =
                is_acceptable(offset_and_image_base, is_64, bad_bytes, good_bytes);
            if acceptable_offset_and_image_base.is_none() && acceptable_offset.is_none(){
                return;
            }
            sender
                .send(format!(
                    "{}\t{:?}\t{:?}",
                    filename, acceptable_offset, acceptable_offset_and_image_base
                ))
                .unwrap();
            return;
        }
    }
}

fn main() {
    let args = Args::parse();
    let bad_bytes = parse_bytes(&args.bad_bytes.unwrap_or(String::new()));
    let good_bytes = parse_bytes(&args.good_bytes.unwrap_or(String::new()));
    let regex = Regex::new(&format!("(?-u){}", &args.regex)).unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut full_files = Vec::new();
    for file in &args.files {
        for entry in glob(file).expect("Failed to read glob pattern") {
            full_files.push(entry.unwrap().to_str().unwrap().to_string());
        }
    }
    full_files.par_iter().for_each_with(tx, |sender, filename| {
        println!("searching {}", filename);
        let mut image_base = 0;
        let mut binary = Vec::new();
        let mut file = File::open(filename).expect("file not found");
        file.read_to_end(&mut binary).unwrap();
        let res = goblin::Object::parse(&binary).unwrap();
        if let goblin::Object::PE(pe) = res {
            if let Some(header) = pe.header.optional_header {
                image_base = header.windows_fields.image_base as usize;
            }
            let sections = pe.sections;
            for mat in regex.find_iter(&binary) {
                verify_search_result(
                    mat,
                    image_base,
                    &sections,
                    &bad_bytes,
                    &good_bytes,
                    sender,
                    filename,
                    pe.is_64,
                );
            }
        } else {
            eprintln!("{:?} is not a PE, skipping", filename);
            return;
        }
    });
    for received in rx {
        println!("{}", received);
    }
    println!("End of search");
}
