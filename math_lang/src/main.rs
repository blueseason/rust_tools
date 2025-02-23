use std::{collections::HashMap, fmt::Display};


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
 
fn main() {
    // swap(pair(a,b)) = pair(b,a)
    use Expr::*;
    let swap = Rule {
        head: Fun("swap".to_string(),
            vec![Fun("pair".to_string(),
                vec![Sym("a".to_string()), Sym("b".to_string())])]),
        body: Fun("pair".to_string(),
            vec![Sym("b".to_string()),Sym("a".to_string())]),
    };


    // Pattern: swap(pair(a,b))
    let pattern = Fun("foo".to_string(),vec![Sym("x".to_string()),Sym("x".to_string())]);
    // Value: swp(pair(f(c),g(d)))
    let value = Fun("foo".to_string(),
        vec![Fun("swap".to_string(),
            vec![Fun("pair".to_string(),
                vec![Fun("f".to_string(),vec![Sym("a".to_string())]),
                    Fun("g".to_string(),vec![Sym("b".to_string())])])]),
            Fun("swap".to_string(),
                vec![Fun("pair".to_string(),
                    vec![Fun("m".to_string(),vec![Sym("c".to_string())]),
                        Fun("n".to_string(),vec![Sym("d".to_string())])])])]);
    println!("Rule:   {}",swap);
    println!("Expr:  {}",value);
    println!("Expr:  {}",swap.apply_all(&value));
    if let Some(bindings) = pattern_match(&pattern,&value){
        println!("MATCH:");
        for (k,v) in bindings.iter() {
            println!("{}  => {}",k,v)
        }
    }else {
        println!("NO MATCH!")
    }
//    println!("{:?}",pattern_match(&pattern,&value));
}
