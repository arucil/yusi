use std::collections::HashMap;
use indexmap::IndexMap;
use crate::grammar::*;

pub(crate) struct Bnf {
  pub(crate) tokens: HashMap<String, TermId>,
  pub(crate) start: IndexMap<String, NontermId>,
  pub(crate) rules: Vec<Nonterm>,
}

pub(crate) enum Symbol {
  Term(TermId),
  Nonterm(NontermId),
}

pub(crate) struct TermId(pub(crate) u32);
pub(crate) struct NontermId(pub(crate) u32);

pub(crate) struct Nonterm {
  pub(crate) name: String,
  pub(crate) prods: Vec<Production>,
}

pub(crate) struct Production {
  pub(crate) symbols: Vec<Symbol>,
}

impl From<Grammar> for Bnf {
  fn from(grammar: Grammar) -> Bnf {
    let tokens = grammar.tokens.into_iter()
      .enumerate()
      .map(|(i, name)| (name, TermId(i as u32)))
      .collect::<HashMap<_, _>>();

  }
}
