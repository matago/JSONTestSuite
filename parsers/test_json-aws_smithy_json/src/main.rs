use std::env;
use std::fs::File;
use std::io::Read;

use aws_smithy_json::deserialize::{json_token_iter, Error, Token};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} file.json", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
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
            let mut open_tokens: Vec<&Token> = Vec::new();
            let mut document_opened = false;
            let mut document_closed = false;
            for token in vectoken.iter() {
                let current_open_token = open_tokens.last();
                match (token, current_open_token) {
                    // We can accept any starting container token, regardless of what we need to close
                    // IF we haven't completed a full container cycle (no forests)
                    (&Token::StartArray { offset: _ } | &Token::StartObject { offset: _ }, _) => {
                        // multi-doc not allowed
                        if document_opened && document_closed {
                            std::process::exit(1)
                        }
                        open_tokens.push(token);
                        document_opened = true;
                    }
                    // We can't accept a closing token that doesnt match
                    (&Token::EndArray { offset: _ }, Some(&Token::StartObject { offset: _ })) | (&Token::EndObject { offset: _ }, Some(&Token::StartArray { offset: _ })) => {
                        std::process::exit(2);
                    }
                    // Becuase of the prior specificity, we can use an OR inside the tuples for the happy path,
                    (&Token::EndArray { offset: _ } | &Token::EndObject { offset: _ }, Some(&Token::StartArray { offset: _ } | &Token::StartObject { offset: _ })) => {
                        open_tokens.pop(); //pull out the matched token
                        if open_tokens.is_empty() {
                            document_closed = true; //we have completed a full container open and closure
                        }
                    }
                    (&Token::EndArray { offset: _ } | &Token::EndObject { offset: _ }, _) => {
                        std::process::exit(1) //closing a container that isnt open
                    }
                    (&Token::ValueString { offset: _, value }, _) => {
                        if value.to_unescaped().is_err() {
                            std::process::exit(1)
                        }
                    }
                    _ => ()
                    // (Token::ObjectKey { offset, key }, None) => todo!(),
                    // (Token::ObjectKey { offset, key }, Some(_)) => todo!(),
                    // (Token::ValueBool { offset, value }, None) => todo!(),
                    // (Token::ValueBool { offset, value }, Some(_)) => todo!(),
                    // (Token::ValueNull { offset }, None) => todo!(),
                    // (Token::ValueNull { offset }, Some(_)) => todo!(),
                    // (Token::ValueNumber { offset, value }, None) => todo!(),
                    // (Token::ValueNumber { offset, value }, Some(_)) => todo!(),
                };
            }
            if !open_tokens.is_empty() {
                std::process::exit(1);
            } // unfinished doc not allowed
        }
    }
    std::process::exit(0);
}
