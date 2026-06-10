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
    let mut parser = rep(heading, None, None);

    let text = Text::new("# Hello\n".to_string().repeat(1));

    let start = Instant::now();
    let res = parser.parse(&text);
    let duration = start.elapsed();

    println!("{:#?}", parser.take_tokens());
    println!("{:?}", res);
    println!("{:?}", duration);
}
