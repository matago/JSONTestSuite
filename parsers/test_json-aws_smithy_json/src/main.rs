use std::env;
use std::fs::File;
use std::io::Read;

use aws_smithy_json::deserialize::{json_token_iter, Error, Token};

#[derive(PartialEq)]
enum ContainerState {
    UnInitiated,
    Open,
    Closed,
}

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
            // No data received
            if vectoken.is_empty() {
                std::process::exit(1)
            }

            let mut container_state = ContainerState::UnInitiated;
            let mut open_tokens: Vec<&Token> = Vec::new();

            for token in vectoken.iter() {
                match (token, open_tokens.last()) {
                    // We can accept any Start* token IF we have not Opened and Closed the first opened container
                    (&Token::StartArray { offset: _ } | &Token::StartObject { offset: _ }, _) => {
                        match container_state {
                            ContainerState::UnInitiated => container_state = ContainerState::Open,
                            ContainerState::Open => (),
                            ContainerState::Closed => std::process::exit(1),
                        }
                        open_tokens.push(token);

                    }
                    // We cannot accept an End{Object,Array} token without a matching Start{Object,Array}
                    (&Token::EndArray { offset: _ }, Some(&Token::StartObject { offset: _ })) | (&Token::EndObject { offset: _ }, Some(&Token::StartArray { offset: _ })) => {
                        std::process::exit(2);
                    }
                    // Because of the prior specificity, we can use an OR inside the tuples for handling the happy path
                    (&Token::EndArray { offset: _ } | &Token::EndObject { offset: _ }, Some(&Token::StartArray { offset: _ } | &Token::StartObject { offset: _ })) => {
                        open_tokens.pop(); //pop the matched token
                        if open_tokens.is_empty() {
                            container_state = ContainerState::Closed
                        }
                    }
                    // We cannot accept End* without a waiting Start* token
                    (&Token::EndArray { offset: _ } | &Token::EndObject { offset: _ }, None) => {
                        std::process::exit(1)
                    }
                    // We cannot accept trailing garabge after closing an outer container
                    (_,_) if container_state == ContainerState::Closed => {
                        std::process::exit(2)
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
