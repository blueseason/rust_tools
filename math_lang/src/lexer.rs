use std::iter::Peekable;

#[derive(Debug)]
enum TokenKind {
    Sym(String),
    OpenParen,
    CloeParen,
    Comma,
    Equals
     
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
//    text: String,
}

pub struct Lexer<Chars: Iterator<Item=char>> {
    chars: Peekable<Chars>
}
 
impl<Chars: Iterator<Item=char>> Lexer<Chars> {
    pub fn from_iter(chars: Chars) -> Self {
        Self {chars: chars.peekable()}   
    }
     /// 辅助函数：读取连续的符号字符，构造完整的符号字符串
    fn lex_symbol(&mut self, first: char) -> String {
        let mut sym = String::new();
        sym.push(first);

        // 根据需要定义什么是符号的一部分，这里举例：字母、数字、下划线
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.chars.next();
                sym.push(ch);
            } else {
                break;
            }
        }
        sym
    }
}
impl<Chars: Iterator<Item=char>> Iterator for Lexer<Chars> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        let token = self.chars.next();
        match token {
            Some(ch) => {
                match ch {
                    '(' => Some(Token {
                        kind: TokenKind::OpenParen,
                      }),
                    ')' => Some(Token {
                        kind: TokenKind::CloeParen,
                      }),
                    ',' => Some(Token {
                        kind: TokenKind::Comma,
                    }),
                    '=' => Some(Token {
                        kind: TokenKind::Equals,
                    }),
                    _ => {
                        Some(Token {
                            kind:TokenKind::Sym(self.lex_symbol(ch)),})   
                    }
                }
            }
            None => None,
        }
    }
}
