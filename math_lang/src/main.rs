use std::{collections::HashMap, fmt::Display, io::{stdin, stdout, Write}, iter::Peekable};
mod lexer;

use lexer::{Token,TokenKind};

use crate::lexer::Lexer;

#[derive(Debug,Clone,PartialEq)]
enum Expr {
    Sym(String),
    Fun(String, Vec<Expr>),
}
enum Error {
    UnexpectedToken(TokenKind, Token),
    UnexpectedEOF(TokenKind),
}

impl Expr {
    // becasue Peekbal Iterator , so need to be mut
    fn parse_peekable(lex: &mut Peekable<impl Iterator<Item=Token>>) -> Result<Self,Error> {
        if let Some(token)  = lex.next() {
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

                        // should not peek
                        if let Some(t) = lex.next() {
                            if t.kind == TokenKind::CloseParen {
                                 Ok(Expr::Fun(token.text, args))
                            }else {
                                 Err(Error::UnexpectedToken(TokenKind::CloseParen, t.clone()))
                            }
                        }else {
                            Err(Error::UnexpectedEOF(TokenKind::CloseParen))
                        }

                    }else {
                        Ok(Expr::Sym(token.text))
                    }                    
                },
                _ => {
                    Err(Error::UnexpectedToken(TokenKind::Sym, token))
                }
            }
        }else {
            Err(Error::UnexpectedEOF(TokenKind::Sym))
        }
    }
    fn parse(lex: impl Iterator<Item=Token>) -> Result<Self,Error> {
        Self::parse_peekable(&mut lex.peekable())
    }
}
#[derive(Debug)]
struct Rule {
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
fn main() {
//    let expr = "swap(pair(pair(c,d), pair(a,b)))";
    let swap = Rule {
        head: expr!(swap(pair(a,b))),
        body: expr!(pair(b,a)),
    };

    let mut command = String::new();
    let prompt = "> ";
    loop {
        command.clear();
        print!("{}",prompt);
        stdout().flush().unwrap();
        stdin().read_line(&mut command).unwrap();
        match Expr::parse(Lexer::from_iter(command.chars())) {
            Ok(expr) => println!("{}",swap.apply_all(&expr)),
            Err(Error::UnexpectedToken(expected, actual)) => {
                println!("{:>width$}^", "", width=prompt.len() + actual.loc.col);
                println!("ERROR: expected {} but got {} '{}'", expected, actual.kind, actual.text)
            }
            Err(Error::UnexpectedEOF(expected)) => {
                println!("{:>width$}^", "", width=prompt.len() + command.len());
                println!("ERROR: expected {} but got nothing", expected)
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
        };

        // Value: swap(foo,swap(pair(f(a),g(b)),swap(pair(m(c),n(d))))
        let input = expr!(
            foo(swap(pair(f(a), g(b))),
                swap(pair(m(c), n(d)))));
        let out  = expr!(
            foo(pair(g(b),f(a)),
                pair(n(d),m(c))));
        assert_eq!(swap.apply_all(&input),out);
    }
}
