use super::*;
use std::time::Instant;

#[test]
fn test_vm() {
    // let mut parser = Parser::new(rep(run(vec![tok("Hello "), tok("Man\n")]), 1, 0));
    // let mut parser = Parser::new(rep(tok("Hello Man\n"), 1, 0));
    let mut parser = Parser::new(rep(tok(till("\n")), 1, 0));
    // let mut parser = Parser::new(rep(
    //     tok(alt(vec![
    //         branch(
    //             run(vec![
    //                 tok(run(vec![rep("#", 1, 6), str(" ")])),
    //                 commit(),
    //                 tok(till("\n")),
    //             ]),
    //             true,
    //         ),
    //         branch(run(vec![tok("> "), commit(), tok(till("\n"))]), true),
    //         branch(till("\n"), false),
    //     ])),
    //     1,
    //     0,
    // ));
    // let mut parser = Parser::new(run(vec![tok("foo"), not("bar")]));

    // let source = "# A Title\n> Quotes!\nHello Man\n".repeat(35);
    let source = "Hello Man\n".repeat(5000000);

    // parser.debug();
    // println!("Program:\n{}", parser.ops.debug_str(true));
    let start = Instant::now();
    let mut res = parser.parse(&source);

    let mut buf = String::new();
    let mut prev_index: u32 = 0;
    while let Some(event) = res.next() {
        if event.start {
            buf.push('(');
        } else {
            buf.push_str(&source[prev_index as usize..event.index as usize]);
            buf.push(')');
            prev_index = event.index;
        }
    }
    let duration = start.elapsed();

    println!("");
    println!("Tokens: {:?}", buf);
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

#[test]
fn test_math() {
    let res: u32 = 5 / 2;
    println!("{res}");
}
