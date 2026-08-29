use super::*;
use std::time::Instant;

#[test]
fn test_vm() {
    // let mut parser = Parser::new(rep(run(vec![tok("Hello "), tok("Man\n")]), 1, 0));
    // let mut parser = Parser::new(rep(tok("Hello Man\n"), 1, 0));
    // let mut parser = Parser::new(rep(tok(till2("\n")), 1, 0));
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

    let source = "# A Title\n> Quotes!\nHello Man\n".repeat(850000);
    // let source = "# A Title\n".repeat(2);

    // parser.debug();
    let start = Instant::now();
    let mut res = parser.parse(&source);

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

    // println!("Program:\n{}", parser.ops.debug_str(true));
    // println!("After Building Events: {:#?}", res);
    // println!("Tokens: {:?}", buf);
    println!("{:?}", parser.stat);
    // println!("Threads: {:#?}", parser.threads);
    println!(
        "Valid/Total Events: {}/{}",
        res.valid_len(),
        res.total_len()
    );
    println!("Total Threads: {}", parser.threads.len());
    println!("Best Match: {:?}", parser.best_match);
    println!("{:?}", duration);
}

#[test]
fn test_compile() {
    let ir = till2("\n");
    println!("{:?}", ir);
    let parser = Parser::new(ir);
    println!("{:?}", parser.ops);
    println!("{}", parser.ops.debug_str(false));
}
