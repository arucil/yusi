use std::collections::HashMap;
use indexmap::IndexMap;
use crate::grammar::*;

pub(crate) struct Bnf {
  pub(crate) tokens: HashMap<String, TermId>,
  pub(crate) start: IndexMap<String, NontermId>,
  pub(crate) nonterms: Vec<Nonterm>,
}

#[derive(Clone)]
pub(crate) enum Symbol {
  Term(TermId),
  Nonterm(NontermId),
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub(crate) struct TermId(pub(crate) u32);

#[derive(Clone, PartialEq, Eq, Copy)]
pub(crate) struct NontermId(pub(crate) u32);

#[derive(Clone, Default)]
pub(crate) struct Nonterm {
  pub(crate) name: String,
  /// non-empty
  pub(crate) prods: Vec<Production>,
}

#[derive(Clone, Default)]
pub(crate) struct Production {
  pub(crate) prec: Option<u16>,
  pub(crate) assoc: Assoc,
  pub(crate) symbols: Vec<Symbol>,
}

impl From<Grammar> for Bnf {
  fn from(grammar: Grammar) -> Bnf {
    let tokens = grammar.tokens.into_iter()
      .enumerate()
      .map(|(i, name)| (name, TermId(i as u32)))
      .collect::<HashMap<_, _>>();

    let mut nonterm_names = HashMap::new();
    for (i, (name, _)) in grammar.rules.iter().enumerate() {
      nonterm_names.insert(name.clone(), NontermId(i as u32));
    }

    let mut nonterms = vec![Nonterm::default(); grammar.rules.len()];
    for (i, (name, rule)) in grammar.rules.into_iter().enumerate() {
      let nonterm = gen_nonterm(
        &mut nonterms, &mut nonterm_names, &tokens, &name, rule);
      nonterms[i] = nonterm;
    }

    let start = grammar.start.into_iter()
      .map(|s| (s, nonterm_names[&s]))
      .collect();

    Bnf {
      tokens,
      start,
      nonterms,
    }
  }
}

fn gen_nonterm(
  nonterms: &mut Vec<Nonterm>,
  nonterm_names: &mut HashMap<String, NontermId>,
  tokens: &HashMap<String, TermId>,
  name: impl Into<String>,
  rule: RuleInner,
) -> Nonterm {
  match rule {
    RuleInner::Sym(_) | RuleInner::Seq(_) | RuleInner::Prec(_) => {
      Nonterm {
        name: name.into(),
        prods: vec![gen_prod(nonterms, nonterm_names, tokens, rule)],
      }
    }
    RuleInner::Or(rules) => {
      Nonterm {
        name: name.into(),
        prods: rules.into_iter()
          .map(|rule| gen_prod(nonterms, nonterm_names, tokens, rule))
          .collect(),
      }
    }
    RuleInner::Many(box RuleRep { rule }) => {
    }
  }
}

fn gen_prod(
  nonterms: &mut Vec<Nonterm>,
  nonterm_names: &mut HashMap<String, NontermId>,
  tokens: &HashMap<String, TermId>,
  rule: RuleInner,
) -> Production {
  match rule {
    RuleInner::Sym(_) => {
    }
  }
}

fn gen_sym(
  nonterms: &mut Vec<Nonterm>,
  nonterm_names: &mut HashMap<String, NontermId>,
  tokens: &HashMap<String, TermId>,
  rule: RuleInner,
) -> Symbol {
  match rule {
    RuleInner::Sym(sym) => {
      if let Some(&term_id) = tokens.get(&sym) {
        Symbol::Term(term_id)
      } else {
        Symbol::Nonterm(nonterm_names[&sym])
      }
    }
  }
}