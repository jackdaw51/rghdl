use std::fs;

use crate::lexer::{Lexer, Token, TokenKind};

pub(crate) mod lexer;

fn main() {
    let source_string = fs::read_to_string("test_files/and_gate.vhd").expect("Not found");

    let mut lexer = Lexer::new(&source_string);

    loop {
        let token = lexer.next_token();

        println!("{:?}", token);

        if token.kind == TokenKind::Eof {
            break;
        }
    }
}
