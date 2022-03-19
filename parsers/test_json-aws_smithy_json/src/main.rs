use std::env;
use std::fs::File;
use std::io::Read;

use aws_smithy_json::deserialize::{json_token_iter, Error, Offset, Token};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} file.json", args[0]);
        std::process::exit(1);
    }

    let ref path = args[1];
    let mut s = String::new();
    let mut f = File::open(path).expect("Unable to open file");
    match f.read_to_string(&mut s) {
        Err(_) => std::process::exit(1),
        Ok(_) => println!("{}", s),
    }

    let result: Result<Vec<Token>, Error> = json_token_iter(s.as_bytes()).collect();
    match result {
        Err(_) => std::process::exit(1),
        Ok(vectoken) => {
            // Unescape all strings and fail if any of them failed to unescape.
            let mut expected_tokens: Vec<Token> = Vec::new();
            let mut document_closed = false;
            for token in vectoken {
                match (token, document_closed) {
                    (Token::StartArray { offset: _ }, false) => {
                        expected_tokens.push(Token::EndArray { offset: Offset(0) })
                    }
                    (Token::StartObject { offset: _ }, false) => {
                        expected_tokens.push(Token::EndObject { offset: Offset(0) })
                    }
                    (Token::EndArray { offset: _ }, false) => {
                        if expected_tokens.pop() != Some(Token::EndArray { offset: Offset(0) }) {
                            std::process::exit(2);
                        };
                        if expected_tokens.is_empty() {
                            document_closed = true;
                        }
                    }
                    (Token::EndObject { offset: _ }, false) => {
                        if expected_tokens.pop() != Some(Token::EndObject { offset: Offset(0) }) {
                            std::process::exit(2);
                        };
                        if expected_tokens.is_empty() {
                            document_closed = true;
                        }
                    }
                    (Token::StartArray { offset: _ } | Token::StartObject { offset: _ }, true) => {
                        std::process::exit(1)
                    } // multi-doc not allowed
                    (Token::ValueString { offset: _, value }, _) => {
                        if value.to_unescaped().is_err() {
                            std::process::exit(1)
                        }
                    }
                    _ => (),
                };
            }
            if !expected_tokens.is_empty() {
                std::process::exit(1);
            } // unfinished doc not allowed
        }
    }
    std::process::exit(0);
}
