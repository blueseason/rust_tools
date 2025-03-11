use std::{collections::HashMap, fmt::Display};
mod lexer;

//use crate::lexer::Lexer;

#[derive(Debug,Clone,PartialEq)]
enum Expr {
    Sym(String),
    Fun(String, Vec<Expr>),
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


macro_rules! sym {
    ($name:ident) => {
        Expr::Sym(stringify!($name).to_string())
    }
}

macro_rules! fun {
    ($name:ident) => {
        Expr::Fun(stringify!($name).to_string(),vec![])
    };
    ($name:ident,$($args:expr),*) => {
        Expr::Fun(stringify!($name).to_string(),vec![$($args),*])
    }
}
fn main() {
    // 
    // for token in Lexer::from_iter("swap(pair(a,b)) = pair(b,a)".chars()) {
    //     println!("{:?}",token);
    // }
    println!("{}",fun!(f));
    println!("{}",fun!(f,sym!(a),sym!(b)));

//    println!("{:?}",pattern_match(&pattern,&value));
}

#[cfg(test)]
mod test {
    use super::*;
    use Expr::*;
    #[test]
    pub fn test_apply_all() {
        let swap = Rule {
            head: fun!(swap,fun!(pair,sym!(a), sym!(b))),
            body: fun!(pair,sym!(b),sym!(a))
        };

        // Value: swap(foo,swap(pair(f(a),g(b)),swap(pair(m(c),n(d))))
        let input = fun!(foo,
            fun!(swap, fun!(pair, fun!(f,sym!(a)), fun!(g,sym!(b)))),
            fun!(swap, fun!(pair, fun!(m,sym!(c)), fun!(n,sym!(d)))));
        let out  = fun!(foo,
            fun!(pair, fun!(g,sym!(b)),  fun!(f,sym!(a))),
            fun!(pair, fun!(n,sym!(d)),  fun!(m,sym!(c))));
        assert_eq!(swap.apply_all(&input),out);
    }
}
