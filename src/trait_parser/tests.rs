use super::super::types::*;
use super::*;
use std::time::Instant;

#[test]
fn test_traits() {
    // let mut parser = Parser::new(tok(
    //     rep(tok(str("Hello Man\n"), Tags::none()), 1, 0),
    //     Tags::none(),
    // ));
    let mut parser = Parser::new(rep(
        chain(&[
            &tok(str("Hello "), Tags::none()),
            &tok(str("Man\n"), Tags::none()),
        ]),
        1,
        0,
    ));
    let source = "Hello Man\n".repeat(100);

    let start = Instant::now();
    let res = parser.parse(&source.as_str());
    let duration = start.elapsed();

    // println!("{:#?}", parser.events);
    println!("{:?}", res);
    println!("{:?}", duration);
}
