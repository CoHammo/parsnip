use super::super::types::*;
use super::*;
use std::time::Instant;

#[test]
fn test_enums() {
    // let mut parser: Parser<u8> = rep(tok(it("Hello Man\n"), Tags::none()), ..);
    let mut parser: Parser<u8> = rep(
        run(vec![
            tok(it("Hello "), Tags::none()),
            tok(it("Man\n"), Tags::none()),
        ]),
        ..,
    );
    // let mut parser: Parser<u8> = rep(tok(till(it("\n"), false), Tags::none()), ..);

    let source = "Hello Man\n".repeat(100);

    let start = Instant::now();
    let res = parser.parse(&source.as_str(), ..);
    let duration = start.elapsed();

    // println!("{:#?}", parser.take_tokens());
    println!("{:?}", res);
    println!("{:?}", duration);
}
