#[cfg(test)]
use super::*;
use std::time::Instant;

#[test] // Marks the function as a test runner
fn main_test() {
    let heading = tok(
        run(vec![
            tok(run(vec![rep(s("#"), Some(1), Some(6)), s(" ")]), None, None),
            tok(till(s("\n"), Some(true)), None, None),
        ]),
        None,
        None,
    );
    let line = tok(till(s("\n"), Some(true)), Some("Line".to_string()), None);

    // let mut parser = tok(rec(s("("), till(s("d"), Some(false)), s(")")), None, None);
    let mut parser = rep(
        tok(
            alt(vec![s("# Hello\n"), s("# Ola\n"), s("# Greetings\n")]),
            None,
            Some(true),
        ),
        None,
        None,
    );

    let text = Text::new("# Hello\n# Ola\n# Greetings\n".to_string().repeat(10000));

    let start = Instant::now();
    let res = parser.parse(&text);
    let duration = start.elapsed();

    let toks = parser.take_tokens();
    // println!("{:#?}", toks);
    println!("{:?}", res);
    println!("{:?} Tokens", toks.map(|t| t.len()));
    println!("{:?}", duration);
}
