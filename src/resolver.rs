use crate::ast::{AstNode, Program};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverDiagnosticKind {
    UnknownIdentifier,
}

impl ResolverDiagnosticKind {
    pub fn message(self) -> &'static str {
        match self {
            ResolverDiagnosticKind::UnknownIdentifier => {
                "Identifier has no declaration in this simplified resolver"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverDiagnostic {
    pub kind: ResolverDiagnosticKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolveResult {
    pub resolved_program: Program,
    pub diagnostics: Vec<ResolverDiagnostic>,
}

#[derive(Default)]
pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, program: &Program) -> ResolveResult {
        let diagnostics = program
            .nodes
            .iter()
            .filter_map(|node| match node {
                AstNode::Identifier(name) => Some(ResolverDiagnostic {
                    kind: ResolverDiagnosticKind::UnknownIdentifier,
                    name: name.clone(),
                }),
                _ => None,
            })
            .collect();

        ResolveResult {
            resolved_program: program.clone(),
            diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_identifier_as_unresolved() {
        let program = Program::new(vec![
            AstNode::Identifier("x".to_owned()),
            AstNode::Number(1.0),
            AstNode::Identifier("y".to_owned()),
        ]);

        let result = Resolver::new().resolve(&program);

        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(result.diagnostics[0].name, "x");
        assert_eq!(result.diagnostics[1].name, "y");
    }
}
