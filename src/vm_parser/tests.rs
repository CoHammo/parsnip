use super::*;
use std::time::Instant;

#[test]
fn test_vm() {
    // let mut parser = Parser::new(rep(run(vec![tok("Hello "), tok("Man\n")]), 1, 0));
    // let mut parser = Parser::new(rep(tok("Hello Man\n"), 1, 0));
    // let mut parser = Parser::new(rep(tok(till("\n")), 1, 0));

    let mut parser = Parser::new(rep(
        alt(vec![
            branch(
                tok(run(vec![
                    tok(run(vec![rep("#", 1, 6), str(" ")])),
                    commit(),
                    tok(till2("\n")),
                ])),
                true,
            ),
            branch(tok(run(vec![tok("> "), commit(), tok(till2("\n"))])), true),
            branch(tok(till2("\n")), false),
        ]),
        1,
        0,
    ));

    // println!("{:?}", parser.comms);
    let source = "# A Title\n> Quotes!\nHello Man\n".repeat(35);

    // parser.debug();
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
    // println!("{:?}", parser.stat);
    println!("{:?}", buf);
    // println!("Threads: {:#?}", parser.threads);
    println!("Total Events: {}", res.total_len());
    // println!("Total Threads: {}", parser.threads.len());
    println!("Best Match: {:?}", parser.best_match);
    println!("{:?}", duration);
}

#[test]
fn test_compile() {
    let parser = Parser::new(not("iter"));

    println!("{:?}", parser.comms);
}
