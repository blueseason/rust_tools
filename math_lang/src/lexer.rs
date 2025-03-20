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

macro_rules! token_kind_enum {
    ($($kind:ident),* $(,)?) => {
        #[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
        pub enum TokenKind {
            $($kind),*
        }

        pub const TOKEN_KIND_ITEMS: [TokenKind; [$(TokenKind::$kind),*].len()] = [$(TokenKind::$kind),*];
    }
}

token_kind_enum! {
    //Rule Keywords
    Rule,
    Shape,
    Apply,
    Done,
    Quit,
    
    Sym,
    OpenParen,
    CloseParen,
    Comma,
    Colon,
    Equals,
    Invalid,
    End,
}

type TokenKindSetInnerType = u64;
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub struct TokenKindSet(TokenKindSetInnerType);

impl TokenKindSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn single(kind: TokenKind) -> Self {
        Self::empty().set(kind)
    }

    pub const fn set(self, kind: TokenKind) -> Self {
        let TokenKindSet(set) = self;
        TokenKindSet(set | (1 << kind as TokenKindSetInnerType))
    }

    pub const fn unset(self, kind: TokenKind) -> Self {
        let TokenKindSet(set) = self;
        TokenKindSet(set & !(1 << kind as TokenKindSetInnerType))
    }

    pub fn contains(&self, kind: TokenKind) -> bool {
        let TokenKindSet(set) = self;
        (set & (1 << kind as u64)) > 0
    }
}

impl std::fmt::Display for TokenKindSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let xs: Vec<TokenKind> = TOKEN_KIND_ITEMS.iter().cloned().filter(|kind| self.contains(*kind)).collect();
        match xs.len() {
            0 => write!(f, "nothing"),
            1 => write!(f, "{}", xs[0]),
            n => {
                write!(f, "{}", xs[0])?;
                for i in 1..n-1 {
                    write!(f, ", {}", xs[i])?
                }
                write!(f, ", or {}", xs[n-1])
            }
        }
    }
}

#[allow(dead_code)]
const TOKEN_KIND_SIZE_ASSERT: [(); (TOKEN_KIND_ITEMS.len() < TokenKindSetInnerType::BITS as usize) as usize] = [()];

fn keyword_by_name(name: &str) -> Option<TokenKind> {
    match name {
        "rule"  => Some(TokenKind::Rule),
        "shape" => Some(TokenKind::Shape),
        "apply" => Some(TokenKind::Apply),
        "done"  => Some(TokenKind::Done),
        "quit"  => Some(TokenKind::Quit),
        _ => None,         
     }

}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        match self {
            Sym => write!(f,"{}","symbol"),
            OpenParen => write!(f, "open paren"),
            CloseParen => write!(f, "close paren"),
            Rule => write!(f, "`rule`"),
            Shape => write!(f, "`shape`"),
            Apply => write!(f, "`apply`"),
            Done => write!(f, "`done`"),
            Quit => write!(f, "`quit`"),
            Comma => write!(f, "comma"),
            Colon => write!(f, "colon"),            
            Equals => write!(f, "equals"),
            Invalid => write!(f, "invalid token"),
            End => write!(f, "end of token"),
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
    exhausted: bool,
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
            exhausted:  false
        }   
    }

    fn loc(&self) -> Loc {
        Loc {
            file_path: self.file_path.clone(),
            row: self.lnum + 1,
            col: self.cnum - self.bol + 1,
        }
    }

    pub fn set_file_path(&mut self, file_path: &str) {
        self.file_path = Some(file_path.to_string())
    }

    fn drop_line(&mut self) {
        while let Some(_) = self.chars.next_if(|x| *x != '\n') {
            self.cnum += 1
        }
        if let Some(_) = self.chars.next_if(|x| *x == '\n') {
            self.cnum += 1;
            self.lnum += 1;
            self.bol = self.cnum
        }
    }

    fn trim_whitespaces(&mut self) {
        while let Some(_) = self.chars.next_if(|x| x.is_whitespace() && *x != '\n') {
            self.cnum += 1
        }
    }
}

impl<Chars: Iterator<Item=char>> Iterator for Lexer<Chars> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        //look ahead use next_if
        if self.exhausted { return None}

        self.trim_whitespaces();
        while let Some(x) = self.chars.peek() {
            if *x != '\n' && *x !='#' {
                break;
            }
            self.drop_line();
            self.trim_whitespaces();
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
                        self.exhausted = true;
                        Some(Token{kind: TokenKind::Invalid, text, loc})
                    }else {
                        while let Some(x) = self.chars.next_if(|x| x.is_alphanumeric()) {
                            self.cnum += 1;
                            text.push(x);
                        }
                        
                        if let Some(kind) = keyword_by_name(&text) {
                            Some(Token{kind, text, loc})
                        } else {
                            Some(Token{kind: TokenKind::Sym, text, loc})
                        }
                    }
                }
            }
        } else {
            self.cnum += 1;
            self.exhausted = true;
            Some(Token{kind: TokenKind::End, text: "".to_string(), loc})
        }
    }
    
}
