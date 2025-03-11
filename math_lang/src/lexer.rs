use std::iter::Peekable;

#[derive(Debug,PartialEq)]
pub enum TokenKind {
    Sym,
    OpenParen,
    CloseParen,
    Comma,
    Equals,  
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

pub struct Lexer<Chars: Iterator<Item=char>> {
    chars: Peekable<Chars>
}
 
impl<Chars: Iterator<Item=char>> Lexer<Chars> {
    pub fn from_iter(chars: Chars) -> Self {
        Self {chars: chars.peekable()}   
    }
}

impl<Chars: Iterator<Item=char>> Iterator for Lexer<Chars> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        //look ahead use next_if
        while let Some(_) = self.chars.next_if(|x| x.is_whitespace()) {}
        if let Some(ch) = self.chars.next() {
            let mut text = String::new();
            text.push(ch);
            match ch {
                '(' => Some(Token { kind: TokenKind::OpenParen, text}),
                ')' => Some(Token { kind: TokenKind::CloseParen, text}),
                ',' => Some(Token { kind: TokenKind::Comma, text}),
                '=' => Some(Token { kind: TokenKind::Equals, text}),
                _ => {
                    if !ch.is_alphanumeric() {
                        todo!("unexpected token properly starts with '{}'",ch);
                    }

                    while let Some(x) = self.chars.next_if(|x| x.is_alphanumeric()) {
                        text.push(x);
                    }
                    
                    Some(Token{kind: TokenKind::Sym, text})
                }
            }
        } else {
            None
        }
    }
}
