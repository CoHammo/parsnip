use super::*;
use std::time::Instant;

#[test]
fn test_vm() {
    // let mut parser = Parser::new(rep(run(vec![tok("Hello "), tok("Man\n")]), 1, 0));
    // let mut parser = Parser::new(rep(tok("Hello Man\n"), 1, 0));
    // let mut parser = Parser::new(rep(tok(till2("\n")), 1, 0));
    let mut parser = Parser::new(rep(
        tok(alt(vec![
            branch(
                run(vec![
                    tok(run(vec![rep("#", 1, 6), str(" ")])),
                    commit(),
                    tok(till("\n")),
                ]),
                true,
            ),
            branch(run(vec![tok("> "), commit(), tok(till("\n"))]), true),
            branch(till("\n"), false),
        ])),
        1,
        0,
    ));
    // let mut parser = Parser::new(rep(tok("Hello Man\n"), 1, 0));

    let source = "# A Title\n> Quotes!\nHello Man\n".repeat(875000);
    // let source = "Hello Man\n".repeat(4500000);

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
    // println!("{:?}", parser.fops.args);
    // println!("After Building Events: {:#?}", res);
    // println!("Tokens: {:?}", buf);
    println!("Stat: {:?}", parser.stat);
    // println!("Threads: {:#?}", parser.threads);
    println!("Total Threads: {}", parser.threads.len());
    println!(
        "Valid/Total Events: {}/{}",
        res.valid_len(),
        res.total_len()
    );
    println!("Best Match: {:?}", parser.best_match);
    println!("{:?}", duration);
}

#[test]
fn test_compiler() {
    let ir = alt(vec![
        branch("Hello", false),
        branch("World", false),
        branch("Man", false),
    ]);
    println!("{:?}", ir);
    let parser = Parser::new(ir);
    println!("{:?}", parser.ops);
    println!("{}", parser.ops.debug_str(false));
}
