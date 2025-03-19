use std::{collections::HashMap, env, fmt::Display, fs, io::{self, stdin, stdout, Write}, iter::Peekable};
mod lexer;

use lexer::*;

#[derive(Debug,Clone,PartialEq)]
enum Expr {
    Sym(String),
    Fun(String, Vec<Expr>),
}
#[derive(Debug)]
enum Error {
    UnexpectedToken(TokenKindSet, Token),
    RuleAlreadyExists(String, Loc, Loc),
    RuleDoesNotExist(String, Loc),
    AlreadyShaping(Loc),
    NoShapingInPlace(Loc),
}

impl Expr {
    // becasue Peekbal Iterator , so need to be mut
    fn parse_peekable(lex: &mut Peekable<impl Iterator<Item=Token>>) -> Result<Self,Error> {
        let token = lex.next().expect("Completely exhausted lexer");
        match token.kind {
            TokenKind::Sym => {
                if let Some(_) = lex.next_if(|t|t.kind == TokenKind::OpenParen){
                    let mut args = Vec::new();
                    if let Some(_) = lex.next_if(|t|t.kind == TokenKind::CloseParen){
                        return Ok(Expr::Fun(token.text,args))
                    }
                    args.push(Self::parse_peekable(lex)?);
                    while let Some(_) = lex.next_if(|t|t.kind == TokenKind::Comma){
                        args.push(Self::parse_peekable(lex)?);
                    }

                    // should not peek because peek do not advance the iterator
                    let close_paren = lex.next().expect("Completely exhausted lexer");
                    if close_paren.kind == TokenKind::CloseParen {
                        Ok(Expr::Fun(token.text, args))
                    }else {
                        Err(Error::UnexpectedToken(TokenKindSet::single(TokenKind::CloseParen),close_paren))
                    }

                }else {
                    Ok(Expr::Sym(token.text))
                }                    
            },
            _ => {
                Err(Error::UnexpectedToken(TokenKindSet::single(TokenKind::Sym), token))
            }
        }
       
    }
    fn parse(lex: impl Iterator<Item=Token>) -> Result<Self,Error> {
        Self::parse_peekable(&mut lex.peekable())
    }
}
#[derive(Debug)]
struct Rule {
    loc: Loc,
    head: Expr,
    body: Expr,
} 

fn subsitute_bindings(bindings: &Bindings, expr: &Expr) -> Expr {
    use Expr::*;
    match expr {
        Sym(name) => {
            if let Some(value) = bindings.get(name) {
                value.clone()
            }else {
                expr.clone()
            }
        },
        Fun(name,args) => {
            let new_name =  match bindings.get(name) {
                Some(Sym(new_name)) => new_name,
                None => name,
                Some(_) => panic!("Expected symbol in the rule"),
            };
            let mut new_args = Vec::new();
            for arg in args {
                new_args.push(subsitute_bindings(bindings,&arg));
            }
            Fun(new_name.to_string(),new_args)
        }
    }
}

impl Rule {
    fn apply_all(&self, expr: &Expr) -> Expr {
        use Expr::*;
        if let Some(bindings) = pattern_match(&self.head,expr) {
            //sub bindings with body
            subsitute_bindings(&bindings,&self.body)
        }else {
            match expr {
                Sym(_) => expr.clone(),
                Fun(name,args)=> {
                    let mut new_args = Vec::new();
                    for arg in args {
                        new_args.push(self.apply_all(arg))
                    }
                    Fun(name.clone(),new_args)
                }
            }
        }
    }
}

type Bindings = HashMap<String,Expr>;

fn pattern_match(pattern:&Expr,value: &Expr) -> Option<Bindings> {
    let mut bindings = HashMap::new();

    fn pattern_match_inner(pattern: &Expr,value: &Expr,bindings:&mut Bindings) -> bool  {
        use Expr::*;
        match (pattern,value) {
            (Sym (name),_) => {
                if let Some(bound_value) = bindings.get(name){
                    bound_value == value
                }else {
                    bindings.insert(name.clone(),value.clone());
                    true   
                }
            },
            (Fun(name1,args1),Fun(name2,args2)) => {
                if name1 == name2 && args1.len() == args2.len() {
                    for i in 0..args1.len() {
                 //       println!("{} - {}",name1,name2);
                        if !pattern_match_inner(&args1[i],&args2[i],bindings) {
                            return false
                        }
                    }
                    true
                }else {
                    false
                }
            },
            _ => false ,
            
        }
    }
    
    if pattern_match_inner(pattern,value,&mut bindings) {
        Some(bindings)
    }else {
        None
    }

}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Sym(name) => write!(f,"{}",name),
            Expr::Fun(name,args) => {
                write!(f,"{}(",name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f,",",)?;}
                    write!(f,"{}",arg)?;
                }
                write!(f,")",)
            },            
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{} = {}", self.head,self.body)
    }
}

macro_rules! fun_args {
    // f()
    () => {vec![]};
    // f(a)
    ($name:ident) => { vec![expr!($name)]};
    // f(a,b,c)
    ($name:ident,$($rest:tt)*) => {
        { //need this block to do macro expension
            let mut t = vec![expr!($name)];
            t.append(& mut fun_args!($($rest)*));
            t
        }
    };
    // f(f(a,b))
    ($name:ident($($args:tt)*)) => {
        vec![expr!($name($($args)*))]
    };

    // f(f(a),...)
    ($name:ident($($args:tt)*),$($rest:tt)*) => {
        {
            let mut t = vec![expr!($name($($args)*))];
            t.append(& mut fun_args!($($rest)*));
            t
        }
    };
}
macro_rules! expr {
    ($name:ident) => {
        Expr::Sym(stringify!($name).to_string())
    };
    //(name,arg1,arg2,...)
    ($name:ident($($args:tt)*)) => {
        Expr::Fun(stringify!($name).to_string(),fun_args!($($args)*))
    };
}

type Rules = HashMap<String,Rule>;

fn expect_token_kind(lexer: &mut Peekable<impl Iterator<Item=Token>>, kind: TokenKindSet) -> Result<Token, Error> {
    let token = lexer.next().expect("Completely exhausted lexer");
    if kind.contains(token.kind) {
        Ok(token)
    } else {
        Err(Error::UnexpectedToken(kind, token))
    }
}


#[derive(Default)]
struct Context {
    rules: HashMap<String, Rule>,
    current_expr: Option<Expr>
}

impl Context {
    fn process_command(&mut self, lexer: &mut Peekable<impl Iterator<Item=Token>>) -> Result<(), Error> {
        let expected_tokens = TokenKindSet::empty()
            .set(TokenKind::Rule)
            .set(TokenKind::Shape)
            .set(TokenKind::Apply)
            .set(TokenKind::Done);
        let keyword = expect_token_kind(lexer, expected_tokens)?;
        match keyword.kind {
            TokenKind::Rule => {
                let name = expect_token_kind(lexer, TokenKindSet::single(TokenKind::Sym))?;
                if let Some(existing_rule) = self.rules.get(&name.text) {
                    return Err(Error::RuleAlreadyExists(name.text, name.loc, existing_rule.loc.clone()))
                }
                let head = Expr::parse(&mut *lexer)?;
                expect_token_kind(lexer, TokenKindSet::single(TokenKind::Equals))?;
                let body = Expr::parse(lexer)?;
                let rule = Rule {
                    loc: keyword.loc,
                    head,
                    body,
                };
                println!("rule {}", &rule);
                self.rules.insert(name.text, rule);
            }
            TokenKind::Shape => {
                if let Some(_) = self.current_expr {
                    return Err(Error::AlreadyShaping(keyword.loc))
                }

                let expr = Expr::parse(lexer)?;
                println!("shaping {}", &expr);
                self.current_expr = Some(expr);
            },
            TokenKind::Apply => {
                if let Some(expr) = &self.current_expr {
                    let expected_kinds = TokenKindSet::empty()
                        .set(TokenKind::Sym)
                        .set(TokenKind::Rule);
                    let token = expect_token_kind(lexer, expected_kinds)?;
                    match token.kind {
                        TokenKind::Sym => {
                            if let Some(rule) = self.rules.get(&token.text) {
                                // todo!("Throw an error if not a single match for the rule was found")
                                let new_expr = rule.apply_all(&expr);
                                println!(" => {}", &new_expr);
                                self.current_expr = Some(new_expr);
                            } else {
                                return Err(Error::RuleDoesNotExist(token.text, token.loc));
                            }
                        }

                        TokenKind::Rule => {
                            let head = Expr::parse(&mut *lexer)?;
                            expect_token_kind(lexer, TokenKindSet::single(TokenKind::Equals))?;
                            let body = Expr::parse(lexer)?;
                            let new_expr = Rule {loc: token.loc, head, body}.apply_all(&expr);
                            println!(" => {}", &new_expr);
                            self.current_expr = Some(new_expr);
                        }

                        _ => unreachable!("Expected {} but got {}", expected_kinds, token.kind)
                    }
                } else {
                    return Err(Error::NoShapingInPlace(keyword.loc));
                }
            }
            TokenKind::Done => {
                if let Some(_) = &self.current_expr {
                    self.current_expr = None
                } else {
                    return Err(Error::NoShapingInPlace(keyword.loc))
                }
            }
            _ => unreachable!("Expected {} but got {}", expected_tokens, keyword.kind),
        }
        Ok(())
    }
}

fn main() {
    let mut args = env::args();
    args.next(); // skip program
    let mut context = Context::default();
    if let Some(file_path) = args.next() {
        let source = fs::read_to_string(&file_path).unwrap();
        let mut lexer = {
            let mut lexer = Lexer::from_iter(source.chars());
            lexer.set_file_path(&file_path);
            lexer.peekable()
        };
        while lexer.peek().expect("Completely exhausted lexer").kind != TokenKind::End {
            if let Err(err) = context.process_command(&mut lexer) {
                match err {
                    Error::UnexpectedToken(expected_kinds, actual_token) => {
                        eprintln!("{}: ERROR: expected {} but got {}", actual_token.loc, expected_kinds, actual_token.kind);
                    }
                    Error::RuleAlreadyExists(name, new_loc, old_loc) => {
                        eprintln!("{}: ERROR: redefinition of existing rule {}", new_loc, name);
                        eprintln!("{}: Previous definition is located here", old_loc);
                    }
                    Error::RuleDoesNotExist(name, loc) => {
                        eprintln!("{}: ERROR: rule {} does not exist", loc, name);
                    }
                    Error::AlreadyShaping(loc) => {
                        eprintln!("{}: ERROR: already shaping an expression. Finish the current shaping with {} first.",
                            loc, TokenKind::Done);
                    }
                    Error::NoShapingInPlace(loc) => {
                        eprintln!("{}: ERROR: no shaping in place.", loc);
                    }
                }
                std::process::exit(1);
            }
        }
    } else {

        let mut command = String::new();
        let prompt = "> ";
        
        loop {
            command.clear();
            print!("{}",prompt);
            stdout().flush().unwrap();
            stdin().read_line(&mut command).unwrap();
            let mut lexer = Lexer::from_iter(command.chars()).peekable();
            let result = context.process_command(&mut lexer)
                .and_then(|()| expect_token_kind(&mut lexer, TokenKindSet::single(TokenKind::End)));
            match result {
                Err(Error::UnexpectedToken(expected, actual)) => {
                    eprintln!("{:>width$}^", "", width=prompt.len() + actual.loc.col);
                    eprintln!("ERROR: expected {} but got {} '{}'", expected, actual.kind, actual.text);
                }
                Err(Error::RuleAlreadyExists(name, new_loc, _old_loc)) => {
                    eprintln!("{:>width$}^", "", width=prompt.len() + new_loc.col);
                    eprintln!("ERROR: redefinition of existing rule {}", name);
                }
                Err(Error::AlreadyShaping(loc)) => {
                    eprintln!("{:>width$}^", "", width=prompt.len() + loc.col);
                    eprintln!("ERROR: already shaping an expression. Finish the current shaping with {} first.",
                        TokenKind::Done);
                }
                Err(Error::NoShapingInPlace(loc)) => {
                    eprintln!("{:>width$}^", "", width=prompt.len() + loc.col);
                    eprintln!("ERROR: no shaping in place.");
                }
                Err(Error::RuleDoesNotExist(name, loc)) => {
                    eprintln!("{:>width$}^", "", width=prompt.len() + loc.col);
                    eprintln!("ERROR: rule {} does not exist", name);
                }
                Ok(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_apply_all() {
        let swap = Rule {
            head: expr!(swap(pair(a,b))),
            body: expr!(pair(b,a)),
        // Value: swap(foo,swap(pair(f(a),g(b)),swap(pair(m(c),n(d))))
        };

        let input = expr!(
            foo(swap(pair(f(a), g(b))),
                swap(pair(m(c), n(d)))));
        let out  = expr!(
            foo(pair(g(b),f(a)),
                pair(n(d),m(c))));
        assert_eq!(swap.apply_all(&input),out);
    }
}
