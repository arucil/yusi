use std::collections::HashMap;
use indexmap::IndexMap;
use crate::grammar::*;

pub(crate) struct Bnf {
  pub(crate) tokens: HashMap<String, TermId>,
  pub(crate) start: IndexMap<String, NontermId>,
  pub(crate) nonterms: Vec<Nonterm>,
}

#[derive(Clone, Copy)]
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
  pub(crate) action: ProdAction,
  pub(crate) prec: Option<u16>,
  pub(crate) assoc: Assoc,
  pub(crate) symbols: Vec<Symbol>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProdAction {
  None,
  /// `rule*  ->  ε`
  StartMany,
  /// `rule*  ->  rule* rule`
  ContinueMany,
  /// `rule+  ->  rule`
  StartSome,
  /// `rule+  ->  rule+ rule`
  ContinueSome,
  /// `rule?  ->  ε`
  EmptyOption,
  /// `rule?  ->  rule`
  NonemptyOption,
  /// `sepBy(sep, rule)  ->  ε`
  EmptySepBy,
  /// `sepBy(sep, rule)  ->  sepBy1(sep, rule)`
  NonemptySepBy,
  /// `sepBy1(sep, rule)  ->  rule`
  StartSepBy1,
  /// `sepBy1(sep, rule)  ->  sepBy1(sep, rule) sep rule`
  ContinueSepBy1,
}

impl Default for ProdAction {
  fn default() -> Self {
    Self::None
  }
}

impl Symbol {
  fn unwrap_nonterm(self) -> NontermId {
    match self {
      Self::Nonterm(id) => id,
      _ => panic!("Term"),
    }
  }
}

impl From<Grammar> for Bnf {
  fn from(grammar: Grammar) -> Bnf {
    let tokens = grammar.tokens.into_iter()
      .enumerate()
      .map(|(i, name)| (name, TermId(i as u32)))
      .collect::<HashMap<_, _>>();

    let mut symbols = HashMap::new();
    for (i, (name, _)) in grammar.rules.iter().enumerate() {
      symbols.insert(name.clone(), Symbol::Nonterm(NontermId(i as u32)));
    }
    for (name, id) in &tokens {
      symbols.insert(name.clone(), Symbol::Term(*id));
    }

    let mut nonterms = vec![Nonterm::default(); grammar.rules.len()];
    for (i, (name, rule)) in grammar.rules.into_iter().enumerate() {
      let id = NontermId(i as u32);
      gen_nonterm(&mut nonterms, &mut symbols, Some(id), &name, rule.0);
    }

    let start = grammar.start.into_iter()
      .map(|s| (s, symbols[&s].unwrap_nonterm()))
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
  symbols: &mut HashMap<String, Symbol>,
  id: Option<NontermId>,
  name: impl Into<String>,
  rule: RuleInner,
) -> NontermId {
  match rule {
    RuleInner::Sym(_) | RuleInner::Seq(_) | RuleInner::Prec(_) => {
      let name = name.into();
      let nonterm = Nonterm {
        name: name.clone(),
        prods: vec![gen_prod(nonterms, symbols, ProdAction::None, rule)],
      };
      if let Some(id@NontermId(ix)) = id {
        nonterms[ix as usize] = nonterm;
        id
      } else {
        let id = NontermId(nonterms.len() as u32);
        symbols.insert(name.clone(), Symbol::Nonterm(id));
        nonterms.push(nonterm);
        id
      }
    }
    RuleInner::Or(rules) => {
      let name = name.into();
      let nonterm = Nonterm {
        name: name.clone(),
        prods: rules.into_iter()
          .map(|rule| gen_prod(nonterms, symbols, ProdAction::None, rule))
          .collect(),
      };
      if let Some(id@NontermId(ix)) = id {
        nonterms[ix as usize] = nonterm;
        id
      } else {
        let id = NontermId(nonterms.len() as u32);
        symbols.insert(name.clone(), Symbol::Nonterm(id));
        nonterms.push(nonterm);
        id
      }
    }
    RuleInner::Many(box RuleRep { rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, move |id, prods| {
        // rule* -> ε
        prods.push(Production {
          action: ProdAction::StartMany,
          ..Default::default()
        });

        // rule* -> rule* rule
        prods.push(Production {
          action: ProdAction::ContinueMany,
          symbols: vec![
            Symbol::Nonterm(id),
            gen_sym(nonterms, symbols, rule),
          ],
          ..Default::default()
        });
      })
    }
    RuleInner::Some(box RuleRep { rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, move |id, prods| {
        let sym = gen_sym(nonterms, symbols, rule);

        // rule+ -> rule
        prods.push(Production {
          action: ProdAction::StartSome,
          symbols: vec![sym],
          ..Default::default()
        });

        // rule+ -> rule+ rule
        prods.push(Production {
          action: ProdAction::ContinueSome,
          symbols: vec![
            Symbol::Nonterm(id),
            sym,
          ],
          ..Default::default()
        });
      })
    }
    RuleInner::Option(box RuleRep { rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, move |id, prods| {
        // rule? -> ε
        prods.push(Production {
          action: ProdAction::EmptyOption,
          ..Default::default()
        });

        // rule? -> rule
        prods.push(gen_prod(nonterms, symbols, ProdAction::NonemptyOption, rule));
      })
    }
  }
}

fn gen_rep_nonterm(
  nonterms: &mut Vec<Nonterm>,
  symbols: &mut HashMap<String, Symbol>,
  id: Option<NontermId>,
  name: impl Into<String>,
  f: impl FnOnce(NontermId, &mut Vec<Production>),
) -> NontermId {
  let name = name.into();
  let id = if let Some(id) = id {
    id
  } else {
    let id = NontermId(nonterms.len() as u32);
    nonterms.push(Nonterm::default());
    id
  };

  symbols.insert(name.clone(), Symbol::Nonterm(id));

  let nonterm = &mut nonterms[id.0 as usize];
  nonterm.name = name;
  f(id, &mut nonterm.prods);

  id
}

fn gen_prod(
  nonterms: &mut Vec<Nonterm>,
  symbols: &mut HashMap<String, Symbol>,
  action: ProdAction,
  rule: RuleInner,
) -> Production {
  match rule {
    RuleInner::Sym(_) => {
    }
  }
}

fn gen_sym(
  nonterms: &mut Vec<Nonterm>,
  symbols: &mut HashMap<String, Symbol>,
  rule: RuleInner,
) -> Symbol {
  match rule {
    RuleInner::Sym(sym) => symbols[&sym],
  }
}