use std::io::{self, Read};



pub struct Separator {
    pub start: char,
    pub end: char,
    pub list_start: char,
    pub list_end: char,
    pub field: char,
    pub item: char,
}

impl Default for Separator {
    fn default() -> Self {
        Separator {
            start: '{',
            end: '}',
            list_start: '[',
            list_end: ']',
            field: ':',
            item: ',',
        }
    }
}

fn main() -> io::Result<()> {
    let mut file = std::fs::File::open("test.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    let chars: Vec<char> = contents.chars().collect();
    let mut tokenized: Vec<String> = Vec::new();
    let mut collect: Vec<char> = Vec::new();
    for c in chars.iter() {
        match c {
            '{' | '}' | '[' | ']' | ':' | ',' | '\n' => {
                    if !collect.is_empty() {
                        let s: String = collect.iter().collect();
                        tokenized.push(s.to_string());
                        collect.clear();
                    }
                    tokenized.push(String::from(*c).to_string());
            }
            ' ' => {
                if collect.is_empty() {
                    tokenized.push(String::from(*c).to_string());
                } else {
                    collect.push(*c);
                }
            }
            _ => collect.push(*c),
        }
    }
    println!("{:?}", tokenized);
/*
    for t in tokenized.iter() {
        println!("{}", t);
    }
*/
    let len = tokenized.len();

    let start = tokenized[0].as_str();

    match start {
        "{" => {
            validate_object(&tokenized, 1);
            }
//        "[" => {
//            validate_array(&tokenized, 1);
//            }
        _ => {
            println!("Invalid JSON");
        }
    }

    Ok(())
}


fn validate_object(tokenized: &Vec<String>, len: usize) {
    let mut i = 1;
    while i < len {
        let key = tokenized[i].as_str();
        let value = tokenized[i + 2].as_str();
        println!("Key: {}", key);
        println!("Value: {}", value);
        i += 4;
    }
}
