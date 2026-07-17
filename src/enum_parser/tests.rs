use super::super::types::*;
use super::*;
use std::time::Instant;

#[test]
fn test_enums() {
    let mut parser: Parser<u8> = run(vec![
        tok(it("Hello "), Tags::none()),
        tok(it("World!\n"), Tags::none()),
    ]);
    // let mut parser = tokker(str("Hello World!\n"), Tags::none());
    let source = "Hello World!\n";

    let start = Instant::now();
    let res = parser.parse(&source, ..);
    // let tree = Node::new(&source, &parser.inner.base().events.take());
    let duration = start.elapsed();

    println!("{:#?}", parser.take_tokens());
    println!("{:?}", res);
    println!("{:?}", duration);
}
