use super::super::types::*;
use super::*;
use std::time::Instant;

#[test]
fn test_traits() {
    let mut parser = tokker(str("Hello World!\n"), Tags::none());
    // let mut parser = tokker(thing("Hello"), Tags::none());
    let source = "Hello World!\n";

    let start = Instant::now();
    let res: Stat;
    let mut iter = source.snips(..);
    while let Some(item) = iter.next() {
        match parser.snip(&item) {
            Stat::Matched(_) | Stat::Failed => break,
            _ => {}
        }
    }
    res = match parser.base().stat {
        Stat::Running => parser.finish(&iter.item()),
        _ => parser.base().stat,
    };
    let tree = Node::new(&source, &parser.base().tokens.take());
    let duration = start.elapsed();

    println!("{:#?}", tree);
    println!("{:?}", res);
    println!("{:?}", duration);
}
