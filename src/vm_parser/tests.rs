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
            tok(till("\n")),
        ]),
        1,
        0,
    ));

    println!("{:?}", parser.comms);
    // let source = "Hello Man\nn".repeat(1);
    let source = "# The Man\n".repeat(100);

    // parser.toggle_debug();
    let start = Instant::now();
    let res = parser.parse(source.as_str());
    let mut res_str = String::new();
    let mut prev_index = 0;
    let mut next_event = parser.first_event;
    while let Some(event_id) = next_event {
        let event = &parser.events[event_id];
        if event.start {
            res_str.push('(');
        } else {
            res_str.push_str(&source[prev_index..event.index]);
            res_str.push_str("),");
        }
        prev_index = event.index;
        next_event = event.next;
    }
    let duration = start.elapsed();

    println!("{:?}", res);
    // println!("Threads: {:#?}", parser.threads);
    // println!("Events: {:#?}", parser.events);
    // println!("{:#?}", parser.ord_events);
    println!("Best Match: {:?}", parser.best_match);
    println!("Total Threads: {}", parser.threads.pool.len());
    println!("{:?}", res_str);
    println!("{:?}", duration);
}

#[test]
fn test_alt_compile() {
    let parser = Parser::new(alt(vec!["hi", "him", "himmel"]));

    println!("{:?}", parser.comms);
}
