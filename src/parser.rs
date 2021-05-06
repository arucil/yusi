use crate::grammar::Grammar;

mod state;
mod bitset;
mod sets;

pub struct Parser {
}

impl Parser {
  pub(crate) fn new(grammar: Grammar) -> Result<Self, String> {
    grammar.validate()?;
    Ok(Parser {})
  }
}