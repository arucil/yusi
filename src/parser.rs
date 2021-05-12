use crate::grammar::Grammar;
use crate::bnf::*;

mod state;
mod bitset;
mod sets;

pub struct Parser {
}

impl Parser {
  pub(crate) fn new(grammar: Grammar) -> Result<Self, String> {
    grammar.validate()?;
    let mut bnf: Bnf = grammar.into();
    bnf.augment();
    self::state::gen_states(&bnf);

    Ok(Parser {})
  }
}