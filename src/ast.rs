#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub nodes: Vec<AstNode>,
}

impl Program {
    pub fn new(nodes: Vec<AstNode>) -> Self {
        Self { nodes }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Identifier(String),
}
