#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::str;
use std::vec::Vec;

////////////////////////////////////////////////////////////////////////////////

pub type IniFile = HashMap<String, HashMap<String, String>>;

pub fn parse(content: &str) -> IniFile {
    let mut answer = IniFile::new();
    let mut header = "";
    let mut flag = false;
    for mut line in content.lines() {
        line = line.trim();
        if line.is_empty() {
            continue;
        }
        let section_opener_flag = line.starts_with('[');
        let section_ender_flag = line.ends_with(']');
        if section_opener_flag || section_ender_flag {
            if !(section_opener_flag && section_ender_flag) {
                panic!("Incorrect section start or finish");
            }
            header = &line[1..line.len() - 1];
            if header.contains('[') || header.contains(']') {
                panic!("Section header should not contain square brackets");
            }
            flag = true;
            answer
                .entry(header.to_string())
                .or_insert_with(HashMap::new);
            continue;
        }
        let tokens: Vec<&str> = line.split('=').collect();
        if tokens.is_empty() {
            panic!("Something is really wrong");
        }
        if tokens.len() > 2 {
            panic!("Only one = allowed");
        }
        if !flag {
            panic!("No section header provided for key value line");
        }
        let key: &str = tokens[0].trim();
        let value: &str = if tokens.len() == 2 {
            tokens[1].trim()
        } else {
            ""
        };
        answer
            .entry(header.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }
    answer
}
