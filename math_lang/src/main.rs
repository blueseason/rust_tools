use std::{collections::HashMap, env, fmt::Display, fs, io::{self, stdin, stdout, Write}, iter::Peekable};
mod lexer;

use lexer::*;

#[derive(Debug,Clone,PartialEq)]
enum Expr {
    Sym(String),
    Var(String),
    //1. change Symbol to Expr to accept Var and Sym, or Fun, etc
    //2. use box becasue of resursive defined Fun with Expr
    Fun(Box<Expr>, Vec<Expr>),
}
#[derive(Debug)]
enum Error {
    UnexpectedToken(TokenKindSet, Token),
    RuleAlreadyExists(String, Loc, Loc),
    RuleDoesNotExist(String, Loc),
    AlreadyShaping(Loc),
    NoShapingInPlace(Loc),
    NoHistory(Loc),
}

impl Expr {
    
    fn var_or_sym_from_name(name: &str) -> Expr {
        if name.chars().next().expect("Empty names are not allowed").is_uppercase() {
            Expr::Var(name.to_string())
        } else {
            Expr::Sym(name.to_string())
        }
    }
    
    // becasue Peekable Iterator , so need to be mut
    fn parse_peekable(lex: &mut Peekable<impl Iterator<Item=Token>>) -> Result<Self,Error> {
        let token = lex.next().expect("Completely exhausted lexer");
        match token.kind {
            TokenKind::Sym => {
                if let Some(_) = lex.next_if(|t|t.kind == TokenKind::OpenParen){
                    let mut args = Vec::new();
                    if let Some(_) = lex.next_if(|t|t.kind == TokenKind::CloseParen){
                        return Ok(Expr::Fun(Box::new(Self::var_or_sym_from_name(&token.text)),args))
                    }
                    args.push(Self::parse_peekable(lex)?);
                    while let Some(_) = lex.next_if(|t|t.kind == TokenKind::Comma){
                        args.push(Self::parse_peekable(lex)?);
                    }

                    // should not peek because peek do not advance the iterator
                    let close_paren = lex.next().expect("Completely exhausted lexer");
                    if close_paren.kind == TokenKind::CloseParen {
                        Ok(Expr::Fun(Box::new(Self::var_or_sym_from_name(&token.text)), args))
                    }else {
                        Err(Error::UnexpectedToken(TokenKindSet::single(TokenKind::CloseParen),close_paren))
                    }

                }else {
                    Ok(Self::var_or_sym_from_name(&token.text))
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
        Sym(_) => expr.clone(),
        Var(name) => {
            if let Some(value) = bindings.get(name) {
                value.clone()
            }else {
                expr.clone()
            }
        },
        Fun(head,args) => {
            let new_head = subsitute_bindings(bindings,head);
            let mut new_args = Vec::new();
            for arg in args {
                new_args.push(subsitute_bindings(bindings,&arg));
            }
            Fun(Box::new(new_head),new_args)
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
                Sym(_) | Var(_) => expr.clone(),
                Fun(head,args)=> {
                    let new_head = self.apply_all(head);
                    let mut new_args = Vec::new();
                    for arg in args {
                        new_args.push(self.apply_all(arg))
                    }
                    Fun(Box::new(new_head),new_args)
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
            (Sym(name1),Sym(name2)) => {
                name1 == name2
            }
            (Var(name),_) => {
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
            Expr::Sym(name) | Expr::Var(name)=> write!(f,"{}",name),
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
    current_expr: Option<Expr>,
    shaping_history: Vec<Expr>,
    quit: bool
}

impl Context {
    fn process_command(&mut self, lexer: &mut Peekable<impl Iterator<Item=Token>>) -> Result<(), Error> {
        let expected_tokens = TokenKindSet::empty()
            .set(TokenKind::Rule)
            .set(TokenKind::Shape)
            .set(TokenKind::Apply)
            .set(TokenKind::Done)
            .set(TokenKind::Undo)
            .set(TokenKind::Quit);
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
                                // just move the expr tp history, no copy
                                self.shaping_history.push(
                                    self.current_expr.replace(new_expr).expect("current_expr must have something")
                                );
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
                            self.shaping_history.push(
                                self.current_expr.replace(new_expr).expect("current_expr must have something")
                            );
                        }

                        _ => unreachable!("Expected {} but got {}", expected_kinds, token.kind)
                    }
                } else {
                    return Err(Error::NoShapingInPlace(keyword.loc));
                }
            }
            TokenKind::Done => {
                if let Some(_) = &self.current_expr {
                    self.current_expr = None;
                    self.shaping_history.clear();
                } else {
                    return Err(Error::NoShapingInPlace(keyword.loc))
                }
            }
            TokenKind::Undo => {
                if let Some(_) = &self.current_expr {
                    if let Some(previous_expr) = self.shaping_history.pop() {
                        println!(" => {}", &previous_expr);
                        self.current_expr.replace(previous_expr);
                    } else {
                        return Err(Error::NoHistory(keyword.loc))
                    }
                } else {
                    return Err(Error::NoShapingInPlace(keyword.loc))
                }
            }
            TokenKind::Quit => {
                self.quit = true;
            }
            _ => unreachable!("Expected {} but got {} '{}'", expected_tokens, keyword.kind,keyword.text),
        }
        Ok(())
    }
}

fn eprint_repl_loc_cursor(prompt: &str, loc: &Loc) {
    assert!(loc.row == 1);
    eprintln!("{:>width$}^", "", width=prompt.len() + loc.col - 1);
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
        while !context.quit && lexer.peek().expect("Completely exhausted lexer").kind != TokenKind::End {
            if let Err(err) = context.process_command(&mut lexer) {
                match err {
                    Error::UnexpectedToken(expected_kinds, actual_token) => {
                        eprintln!("{}: ERROR: expected {} but got {} '{}'", actual_token.loc, expected_kinds, actual_token.kind,actual_token.text);
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
                    Error::NoHistory(loc) => {
                        eprintln!("{}: ERROR: no history", loc);
                    }
                }
                std::process::exit(1);
            }
        }
    } else {

        let mut command = String::new();
        let prompt = "> ";
        
        while !context.quit {
            command.clear();
            print!("{}",prompt);
            stdout().flush().unwrap();
            stdin().read_line(&mut command).unwrap();
            let mut lexer = Lexer::from_iter(command.chars()).peekable();
            let result = context.process_command(&mut lexer)
                .and_then(|()| expect_token_kind(&mut lexer, TokenKindSet::single(TokenKind::End)));
            match result {
                Err(Error::UnexpectedToken(expected, actual)) => {
                    eprint_repl_loc_cursor(prompt, &actual.loc);
                    eprintln!("ERROR: expected {} but got {} '{}'", expected, actual.kind, actual.text);
                }
                Err(Error::RuleAlreadyExists(name, new_loc, _old_loc)) => {
                    eprint_repl_loc_cursor(prompt, &new_loc);
                    eprintln!("ERROR: redefinition of existing rule {}", name);
                }
                Err(Error::AlreadyShaping(loc)) => {
                    eprint_repl_loc_cursor(prompt,&loc);
                    eprintln!("ERROR: already shaping an expression. Finish the current shaping with {} first.",
                        TokenKind::Done);
                }
                Err(Error::NoShapingInPlace(loc)) => {
                    eprint_repl_loc_cursor(prompt,&loc);
                    eprintln!("ERROR: no shaping in place.");
                }
                Err(Error::RuleDoesNotExist(name, loc)) => {
                    eprint_repl_loc_cursor(prompt,&loc);
                    eprintln!("ERROR: rule {} does not exist", name);
                }
                Err(Error::NoHistory(loc)) => {
                    eprintln!("{}: ERROR: no history", loc);
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
            loc: Loc{file_path:None,row:0,col:0},
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
