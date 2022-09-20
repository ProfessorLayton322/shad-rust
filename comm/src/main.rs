#![forbid(unsafe_code)]

use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let firstFile = std::fs::File::open(&args[1]).unwrap();
    let mut reader = BufReader::new(firstFile);

    let mut firstSet = HashSet::new();

    for line in reader.lines() {
        firstSet.insert(line.unwrap());
    }

    let secondFile = std::fs::File::open(&args[2]).unwrap();
    reader = BufReader::new(secondFile);

    let mut secondSet = HashSet::new();

    for line in reader.lines() {
        secondSet.insert(line.unwrap());
    }

    let mut intersection = firstSet.intersection(&secondSet);
    for line in intersection {
        println!("{}", line);
    }
}
