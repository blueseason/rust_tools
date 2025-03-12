use std::iter::Peekable;
use std::fmt::Display;

#[derive(Debug,Clone)]
pub struct Loc {
    pub file_path: Option<String>,
    pub row: usize,
    pub col: usize,
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.file_path {
            Some(file_path) => write!(f, "{}:{}:{}", file_path, self.row, self.col),
            None => write!(f, "{}:{}", self.row, self.col),
        }
    }
}


#[derive(Debug,PartialEq,Clone)]
pub enum TokenKind {
    Sym,
    OpenParen,
    CloseParen,
    Comma,
    Colon,
    Equals,
    Invalid,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        match self {
            Sym => write!(f,"{}","symbol"),
            OpenParen => write!(f, "open paren"),
            CloseParen => write!(f, "close paren"),
            Comma => write!(f, "comma"),
            Colon => write!(f, "colon"),            
            Equals => write!(f, "equals"),
            Invalid => write!(f, "invalid token"),
        }
    }
}

#[derive(Debug,Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub loc: Loc,
}

pub struct Lexer<Chars: Iterator<Item=char>> {
    chars: Peekable<Chars>,
    invalid: bool,
    file_path: Option<String>,
    lnum: usize,
    bol: usize,
    cnum: usize,
}
 
impl<Chars: Iterator<Item=char>> Lexer<Chars> {
    pub fn from_iter(chars: Chars) -> Self {
        Self {
            chars: chars.peekable(),
            invalid: false,
            file_path: None,
            lnum: 0,
            bol: 0,
            cnum: 0,
        }   
    }

    fn loc(&self) -> Loc {
        Loc {
            file_path: self.file_path.clone(),
            row: self.lnum,
            col: self.cnum - self.bol,
        }
    }

    pub fn set_file_path(&mut self, file_path: &str) {
        self.file_path = Some(file_path.to_string())
    }
}

impl<Chars: Iterator<Item=char>> Iterator for Lexer<Chars> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        //look ahead use next_if
        if self.invalid { return None}
        
        while let Some(x) = self.chars.next_if(|x| x.is_whitespace()) {
            self.cnum += 1;
            if  x == '\n' {
                self.lnum += 1;
                self.bol = self.cnum;
            }
        }
        let loc = self.loc();
        self.cnum += 1;
        if let Some(ch) = self.chars.next() {
            let mut text = String::new();
            text.push(ch);
            match ch {
                '(' => Some(Token { kind: TokenKind::OpenParen, text, loc}),
                ')' => Some(Token { kind: TokenKind::CloseParen, text, loc}),
                ',' => Some(Token { kind: TokenKind::Comma, text, loc}),
                '=' => Some(Token {kind: TokenKind::Equals, text, loc}),
                ':' => Some(Token {kind: TokenKind::Colon, text, loc}),                
                _ => {
                    if !ch.is_alphanumeric() {
                        self.invalid = true;
                        Some(Token{kind: TokenKind::Invalid, text, loc})
                    }else {
                        while let Some(x) = self.chars.next_if(|x| x.is_alphanumeric()) {
                            self.cnum += 1;
                            text.push(x);
                        }
                        Some(Token{kind: TokenKind::Sym, text, loc})
                    }
                }
            }
        } else {
            None
        }
    }
}
