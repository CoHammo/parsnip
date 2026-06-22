use super::{Tag, Tags, Token};

#[derive(Debug)]
pub struct Node<'a> {
    pub tag: Tag,
    pub value: &'a str,
    pub children: Option<Vec<Node<'a>>>,
}

impl<'a> Node<'a> {
    pub fn new(source: &'a str, tokens: &Option<Vec<Token>>) -> Self {
        Self {
            tag: Tags::none(),
            value: source,
            children: match tokens {
                Some(tokens) => Some(tokens.iter().map(|t| Node::make(source, t)).collect()),
                None => None,
            },
        }
    }

    pub fn make(source: &'a str, token: &Token) -> Self {
        Self {
            tag: token.tag,
            value: &source[token.start..token.end],
            children: match &token.tokens {
                Some(tokens) => Some(tokens.iter().map(|t| Node::make(source, t)).collect()),
                None => None,
            },
        }
    }

    pub fn len(&self) -> usize {
        match &self.children {
            Some(children) => children.len(),
            None => 0,
        }
    }
}
