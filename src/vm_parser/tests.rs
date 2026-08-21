use super::*;
use std::time::Instant;

#[test]
fn test_vm() {
    // let mut parser = Parser::new(rep(run(vec![tok("Hello "), tok("Man\n")]), 1, 0));
    // let mut parser = Parser::new(rep(tok("Hello Man\n"), 1, 0));
    // let mut parser = Parser::new(rep(tok(till("\n")), 1, 0));
    // let mut parser = Parser::new(rep(alt(vec![tok("Hello "), tok("World!\n")]), 1, 0));

    let mut parser = Parser::new(rep(
        alt(vec![
            tok(run(vec![
                tok(run(vec![rep("#", 1, 6), str(" ")])),
                tok(till("\n")),
            ])),
            // tok(till("\n")),
        ]),
        1,
        0,
    ));

    println!("{:?}", parser.comms);
    // let source = "Hello Man\nn".repeat(1);
    let source = "# The Man\n".repeat(100);

    // parser.toggle_debug();
    let start = Instant::now();
    let mut res = parser.parse(source.as_str());

    let mut buf = String::new();
    let mut prev_index = 0;
    while let Some(event) = res.next() {
        if event.start {
            buf.push('(');
        } else {
            buf.push_str(&source[prev_index..event.index]);
            buf.push(')');
            prev_index = event.index;
        }
    }
    let duration = start.elapsed();

    // println!("After Building Events: {:#?}", res);
    println!("{:?}", parser.stat);
    // println!("Threads: {:#?}", parser.threads);
    // println!("Events: {:#?}", parser.events);
    // println!("{:#?}", parser.ord_events);
    println!("Best Match: {:?}", parser.best_match);
    println!("Total Threads: {}", parser.threads.len());
    println!("{:?}", buf);
    println!("{:?}", duration);
}

#[test]
fn test_alt_compile() {
    let parser = Parser::new(alt(vec!["hi", "him", "himmel"]));

    println!("{:?}", parser.comms);
}
