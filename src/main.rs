use clap::{load_yaml, App};
use std::fs;
extern crate regex;
use regex::Regex;

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
struct SpartanExpressionError {
    err: String,
}

fn str_2_spartan(
    tex_string: String,
    mut ctx: Option<Vec<String>>,
) -> Result<SpartanExpression, SpartanExpressionError> {
    println!("parsing {}", tex_string);

    let tokens = tex_string.split(" ").collect::<Vec<&str>>();
    match tokens[..] {
        [] => {
            return Err(SpartanExpressionError {
                err: "No tokens".to_string(),
            })
        }
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
                    println!("Found value, {:?}", tokens);
                    let bracket_re = Regex::new(r"\([a-z A-Z()1-9]+\)").expect("A regex");
                    let b_split: Vec<&str> = tex_string.splitn(2, "(").collect();
                    if b_split.len() != 1 {
                        // Brackets in expression
                        let b_exps = b_split
                            .into_iter()
                            .filter(|e| e.contains("(") || e.contains(")"))
                            .map(|e| &e[0..e.len() - 1])
                            .collect::<Vec<&str>>();
                        let b_exps: Vec<Vec<&str>> = b_exps
                            .into_iter()
                            .map(|e| e.splitn(2, " ").collect())
                            .collect();
                        let mut b_exps_2 = Vec::new();
                        b_exps.into_iter().for_each(|e| b_exps_2.extend(e));
                        b_exps_2 = b_exps_2.into_iter().filter(|e| *e != "").collect();
                        println!("Detected bracketed expression: {:?}", b_exps_2);

                        let e1;
                        let e2;
                        e1 = Box::new(
                            str_2_spartan(b_exps_2[0].to_string(), ctx.clone())
                                .expect("Expression 1"),
                        );
                        e2 = Box::new(
                            str_2_spartan(b_exps_2[1].to_string(), ctx.clone())
                                .expect("Expression 2"),
                        );
                        let ret = SpartanExpression::APP(e1, e2);
                        return Ok(ret);
                    }
                    let tokens: Vec<String> = tokens
                        .into_iter()
                        .map(|t| t.replace(")", ""))
                        .filter(|t| *t != "")
                        .collect();
                    println!("parsing value, {:?}", tokens);
                    if tokens.len() == 1 {
                        println!("Single value detected, {}", tokens[0]);
                        if ctx.unwrap_or(Vec::new()).contains(&tokens[0].to_string()) {
                            println!("found variable");
                            return Ok(SpartanExpression::Var(tokens[0].to_string()));
                        } else {
                            return Ok(SpartanExpression::Val(
                                tokens[0].parse::<i32>().expect("Vals should be i32"),
                            ));
                        }
                    } else {
                        let e1;
                        let e2;
                        e1 = Box::new(
                            str_2_spartan(tokens[0].to_string(), ctx.clone())
                                .expect("Expression 1"),
                        );
                        e2 = Box::new(
                            str_2_spartan(tokens[1].to_string(), ctx.clone())
                                .expect("Expression 2"),
                        );
                        let mut ret = SpartanExpression::APP(e1, e2);
                        if tokens.len() > 1 {
                            for i in 2..tokens.len() {
                                let e2;
                                e2 = Box::new(
                                    str_2_spartan(tokens[i].to_string(), ctx.clone())
                                        .expect("Expression 2"),
                                );
                                ret = SpartanExpression::APP(Box::new(ret), e2);
                            }
                        }
                        return Ok(ret);
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
