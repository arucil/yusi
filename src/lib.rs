#![feature(box_syntax, box_patterns, bindings_after_at)]

pub mod grammar;
pub mod parser;
mod bnf;

pub use parser::Parser;
pub use grammar::Grammar;

pub fn build(grammar: Grammar) -> Result<Parser, String> {
  Parser::new(grammar)
}
