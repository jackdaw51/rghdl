// Let's make this shi allocation free

use std::{
    iter::Peekable,
    ops::{AddAssign, Index},
    str::Chars,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
impl Token {
    fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Identifier,
    Number,       // 16#FF#, 3.14
    StringLit,    // "Marco"
    CharLit,      // '1', 'Z'
    BitStringLit, // x"FF", b"1010"

    KwEntity,
    KwArchitecture,
    KwPackage,
    KwIs,
    KwPort,
    KwGeneric,
    KwBegin,
    KwEnd,
    KwProcess,
    KwIf,
    KwThen,
    KwElse,
    KwLibrary,
    KwUse,
    KwAll,

    OpAssign,            // :=
    OpArrow,             // => (Port mapping)
    OpSignalAssignOrLEq, // <= Signal assignment or less equal
    OpEq,                // =
    OpNeq,               // /=
    OpLt,                // <
    OpGt,                // >
    OpGeq,               // >=
    OpBox,               // <> (Unconstrained range)
    Colon,               // :
    Semicolon,           // ;
    Comma,               // ,
    Dot,                 // .
    Tick,                // '
    LParen,              // (
    RParen,              // )

    Eof,
    Error,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current_pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            current_pos: 0,
        }
    }

    /// The main method called by the Parser
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start_pos = self.current_pos;

        // Peek at the next character to determine the token type
        if let Some(&ch) = self.chars.peek() {
            print!("{ch} ");
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' => self.lex_identifier_or_keyword(start_pos),
                ':' | '<' => self.lex_two_char(start_pos),
                '-' => self.lex_minus_or_comment(start_pos),
                ';' | '.' | '(' | ')' => self.single_digit(start_pos),
                _ => self.lex_unknown(start_pos),
            }
        } else {
            Token {
                kind: TokenKind::Eof,
                span: Span {
                    start: start_pos,
                    end: start_pos,
                },
            }
        }
    }
    /// Consumes the next character and updates the byte offset
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            // ch.len_utf8() ensures we don't panic on multibyte characters
            self.current_pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let remaining = &self.source[self.current_pos..];

            if remaining.starts_with("--") {
                self.skip_line_comment();
                continue; // Loop back around to check for more whitespace/comments
            }

            if remaining.starts_with("/*") {
                self.skip_block_comment();
                continue;
            }

            if let Some(&ch) = self.chars.peek() {
                if ch.is_whitespace() {
                    self.advance();
                    continue;
                }
            }

            break;
        }
    }

    /// Skips characters until a newline is found
    fn skip_line_comment(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch == '\n' {
                break; // Leave the newline to be eaten by the whitespace checker
            }
            self.advance();
        }
    }

    /// Skips characters until "*/" is found (Optional for VHDL-2019 support)
    fn skip_block_comment(&mut self) {
        // Eat the initial "/*" so we don't immediately trigger on it
        self.advance();
        self.advance();

        while let Some(_) = self.chars.peek() {
            let remaining = &self.source[self.current_pos..];
            if remaining.starts_with("*/") {
                self.advance(); // Eat '*'
                self.advance(); // Eat '/'
                break;
            }
            self.advance();
        }
    }
    fn lex_identifier_or_keyword(&mut self, start_pos: usize) -> Token {
        while let Some(c) = self.chars.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.advance();
                }
                _ => break,
            }
        }

        let a = match &self.source[start_pos..self.current_pos] {
            "library" => TokenKind::KwLibrary,
            _ => TokenKind::Identifier,
        };

        Token {
            kind: a,
            span: Span::new(start_pos, self.current_pos),
        }
    }

    fn lex_number(&self, start_pos: usize) -> Token {
        todo!()
    }

    fn lex_two_char(&mut self, start_pos: usize) -> Token {
        let t;
        let iter_clone = self.chars.clone().skip(1).next();

        let Some(first_c) = self.chars.peek() else {
            return self.error(start_pos);
        };

        println!("{first_c}");
        let Some(second_c) = iter_clone else {
            return self.error(start_pos);
        };

        t = match second_c {
            '=' => match first_c {
                '<' => TokenKind::OpSignalAssignOrLEq,
                ':' => TokenKind::OpAssign,
                '/' => TokenKind::OpNeq,
                '>' => TokenKind::OpGeq,
                _ => unreachable!(),
            },
            '>' => match first_c {
                '=' => TokenKind::OpArrow,
                '<' => TokenKind::OpBox,
                _ => unreachable!(),
            },
            _ => return self.single_digit(start_pos),
        };
        self.advance();
        self.advance();
        Token::new(t, Span::new(start_pos, self.current_pos))
    }

    fn error(&self, start_pos: usize) -> Token {
        Token::new(TokenKind::Error, Span::new(start_pos, self.current_pos))
    }

    fn lex_minus_or_comment(&self, start_pos: usize) -> Token {
        todo!()
    }

    fn lex_unknown(&self, start_pos: usize) -> Token {
        todo!()
    }

    fn single_digit(&mut self, start_pos: usize) -> Token {
        let mut t = TokenKind::Error;
        if let Some(c) = self.advance() {
            t = match c {
                '.' => TokenKind::Dot,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                ':' => TokenKind::Semicolon,
                '<' => TokenKind::OpLt,
                _ => unreachable!(),
            }
        }
        Token::new(t, Span::new(start_pos, self.current_pos))
    }
}
