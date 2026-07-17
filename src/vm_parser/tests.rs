use super::*;
use std::time::Instant;

#[test]
fn test_vm() {
    let mut parser = Parser::new(tok(run(vec![tok("Hello "), tok("World!\n")])));
    // let mut parser = Parser::new(tok("Hello World!\n"));
    println!("{:?}", parser.comms);
    let source = "Hello World!\n".repeat(1);

    let start = Instant::now();
    let res = parser.parse(source.as_str());
    let duration = start.elapsed();

    println!("{:?}", res);
    println!("{:#?}", parser.events);
    println!("{:?}", parser.matches);
    println!("{:?}", duration);
}
