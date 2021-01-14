use clap::{load_yaml, App};
use std::fs;
extern crate regex;

#[derive(Debug)]
enum SpartanExpression {
    Val(i32),
    Var(String),
    LAMBDA(Box<SpartanExpression>, Box<SpartanExpression>),
    APP(Box<SpartanExpression>, Box<SpartanExpression>),
}

impl std::fmt::Display for SpartanExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            SpartanExpression::LAMBDA(v, e) => write!(f, "{}", format!("LAMBDA(;{}.{})", *v, *e)),
            SpartanExpression::Val(v) => write!(f, "{}", format!("{}", v)),
            SpartanExpression::Var(v) => write!(f, "{}", v),
            SpartanExpression::APP(v1, v2) => write!(f, "{}", format!("APP({},{})", *v1, *v2)),
        }
    }
}

#[derive(Debug)]
struct SpartanExpressionError;
fn str_2_spartan(
    tex_string: String,
    mut ctx: Option<Vec<String>>,
) -> Result<SpartanExpression, SpartanExpressionError> {
    println!("parsing {}", tex_string);
    let mut tokens = tex_string.split(" ").collect::<Vec<&str>>();
    match tokens[..] {
        [] => return Err(SpartanExpressionError),
        _ => {
            println!("tokens {:?}", tokens);
            println!("context {:?}", ctx);
            // detect brackets

            match tokens[0] {
                "\\lambda" => {
                    println!("Found lambda");
                    let mut new_ctx: Vec<String> = vec![String::from(tokens[1].replace(".", ""))];
                    if ctx != None {
                        new_ctx.extend(ctx.expect("old ctx"));
                    }
                    ctx = Some(new_ctx);
                    let ret = Ok(SpartanExpression::LAMBDA(
                        Box::new(SpartanExpression::Var(
                            tokens[1].replace(".", "").to_string(),
                        )),
                        Box::new(str_2_spartan(
                            tokens[2..]
                                .iter()
                                .fold(String::new(), |acc, &l| acc + " " + l)[1..]
                                .to_string(),
                            ctx,
                        )?),
                    ));
                    return ret;
                }
                _ => {
                    println!("Found value");
                    println!("Unpacking vars, ctx: {:?}, \n tokens: {:?}", ctx, tokens);
                    let bracketed_tokens: Vec<&str> = tokens
                        .clone()
                        .into_iter()
                        .filter(|t| t.contains("(") || t.contains(")"))
                        .collect();
                    println!("Bracketed tokens: {:?}", bracketed_tokens);
                    let bracket_string = &bracketed_tokens
                        .iter()
                        .fold(String::new(), |acc, &l| acc + " " + l);
                    let n = bracket_string.len();

                    if n != 0 {
                        let bracket_expr =
                            str_2_spartan(bracket_string[2..n - 1].to_string(), ctx.clone());
                        println!("Bracketed expr: {:?}", bracket_expr);
                    }
                    tokens = tokens
                        .into_iter()
                        .filter(|t| !(t.contains("(") || t.contains(")")) && *t != "")
                        .collect();

                    match ctx {
                        None => return Err(SpartanExpressionError),
                        Some(ctx) => {
                            let e1;
                            let e2;
                            if ctx.contains(&tokens[0].to_string()) {
                                e1 = Box::new(SpartanExpression::Var(tokens[0].to_string()));
                            } else {
                                e1 = Box::new(SpartanExpression::Val(
                                    tokens[0].parse::<i32>().expect("Vals should be i32"),
                                ));
                            }
                            if ctx.contains(&tokens[1].to_string()) {
                                e2 = Box::new(SpartanExpression::Var(tokens[1].to_string()));
                            } else {
                                e2 = Box::new(SpartanExpression::Val(
                                    tokens[1].parse::<i32>().expect("Vals should be i31"),
                                ));
                            }
                            let mut ret = SpartanExpression::APP(e1, e2);
                            for i in 2..tokens.len() {
                                let e2;
                                if ctx.contains(&tokens[i].to_string()) {
                                    e2 = Box::new(SpartanExpression::Var(tokens[i].to_string()));
                                } else {
                                    e2 = Box::new(SpartanExpression::Val(
                                        tokens[i].parse::<i32>().expect("Vals should be i32"),
                                    ));
                                }
                                ret = SpartanExpression::APP(Box::new(ret), e2);
                            }
                            return Ok(ret);
                        }
                    }
                }
            }
        }
    };
}

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let mut latex_expression = String::from("\\lambda f. \\lambda x. f x");
    // LAMBDA(; f. LAMBDA(; x. APP(f x)))
    if let Some(f) = matches.value_of("file") {
        latex_expression = fs::read_to_string(f).expect("File contents");
    } else if let Some(s) = matches.value_of("string") {
        latex_expression = String::from(s);
    }

    let res = str_2_spartan(latex_expression, None);
    println!("{}", res.unwrap());
}
