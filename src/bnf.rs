use std::collections::HashMap;
use std::ops::Range;
use indexmap::IndexMap;
use crate::grammar::*;

#[derive(Debug)]
pub(crate) struct Bnf {
  pub(crate) tokens: IndexMap<String, TermId>,
  pub(crate) starts: IndexMap<String, NontermId>,
  pub(crate) nonterms: Vec<Nonterm>,
  pub(crate) prods: Vec<Production>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Symbol {
  Term(TermId),
  Nonterm(NontermId),
}

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub(crate) struct TermId(pub(crate) u32);

#[derive(Clone, PartialEq, Eq, Copy, Debug, Hash, Default)]
pub(crate) struct NontermId(pub(crate) u32);

#[derive(Clone, Default, Debug)]
pub(crate) struct Nonterm {
  pub(crate) name: String,
  /// non-empty
  pub(crate) prod_range: Range<usize>,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct Production {
  pub(crate) nonterm_id: NontermId,
  pub(crate) action: ProdAction,
  pub(crate) prec: Option<u16>,
  pub(crate) assoc: Assoc,
  pub(crate) symbols: Vec<Symbol>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ProdAction {
  None,
  /// `rule*  ->  ε`
  StartMany,
  /// `rule*  ->  rule* rule`
  ContinueMany,
  /// `rule+  ->  rule`
  StartMany1,
  /// `rule+  ->  rule+ rule`
  ContinueMany1,
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

impl Bnf {
  pub(crate) fn augment(&mut self) {
    for (name, id) in &mut self.starts {
      let new_id = NontermId(self.nonterms.len() as u32);
      self.nonterms.push(Nonterm {
        name: format!("$start({})", name),
        prod_range: self.prods.len() .. self.prods.len() + 1,
      });
      self.prods.push(Production {
        nonterm_id: new_id,
        symbols: vec![Symbol::Nonterm(*id)],
        ..Default::default()
      });

      *id = new_id;
    }
  }

  #[cfg(test)]
  pub(crate) fn parse(input: &str) -> Self {
    let mut start_nts = indexmap::IndexSet::new();
    let rules = input.trim().lines()
      .map(|line| {
        let line = line.split("->").collect::<Vec<_>>();
        (line[0].trim(), line[1].trim())
      })
      .map(|(mut lhs, rhs)| {
        if lhs.ends_with("*") {
          lhs = &lhs[..lhs.len() - 1];
          start_nts.insert(lhs);
        }
        let symbols = rhs.split_ascii_whitespace()
          .map(|sym| sym.trim())
          .collect::<Vec<_>>();
        (lhs, symbols)
      })
      .fold(IndexMap::<_, Vec<_>>::new(), |mut rules, (lhs, rhs)| {
        rules.entry(lhs).or_default().push(rhs);
        rules
      });

    let mut starts = start_nts.into_iter()
      .map(|nt|
        (nt.to_owned(), NontermId(rules.get_index_of(nt).unwrap() as u32)))
      .collect();

    let mut tokens = IndexMap::<String, TermId>::new();
    let mut nonterms: Vec<Nonterm> = vec![];
    let mut prods: Vec<Production> = vec![];

    for (nt, rule) in &rules {
      let nonterm_id = NontermId(nonterms.len() as u32);
      nonterms.push(Nonterm {
        name: (*nt).to_owned(),
        prod_range: prods.len() .. prods.len() + prods.len(),
      });
      for symbols in rule {
        prods.push(Production {
          nonterm_id,
          symbols: symbols.iter()
            .map(|sym| {
              if let Some(ix) = rules.get_index_of(sym) {
                Symbol::Nonterm(NontermId(ix as u32))
              } else if let Some(id) = tokens.get(*sym) {
                Symbol::Term(*id)
              } else {
                let id = TermId(tokens.len() as u32);
                tokens.insert((*sym).to_owned(), id);
                Symbol::Term(id)
              }
            })
            .collect(),
          ..Default::default()
        });
      }
    }

    Bnf { tokens, starts, nonterms, prods }
  }
}

impl From<Grammar> for Bnf {
  fn from(grammar: Grammar) -> Bnf {
    let tokens = grammar.tokens.into_iter()
      .enumerate()
      .map(|(i, name)| (name, TermId(i as u32)))
      .collect::<IndexMap<_, _>>();

    let mut symbols = HashMap::new();
    for (i, (name, _)) in grammar.rules.iter().enumerate() {
      symbols.insert(name.clone(), Symbol::Nonterm(NontermId(i as u32)));
    }
    for (name, id) in &tokens {
      symbols.insert(name.clone(), Symbol::Term(*id));
    }

    let mut nonterms = vec![Nonterm::default(); grammar.rules.len()];
    let mut prods = vec![];
    for (name, rule) in grammar.rules {
      gen_nonterm(&mut nonterms, &mut prods, &mut symbols, &name, rule.0);
    }

    let starts = grammar.start.into_iter()
      .map(|s| {
        let id = symbols[&s].unwrap_nonterm();
        (s, id)
      })
      .collect();

    Bnf {
      tokens,
      starts,
      nonterms,
      prods,
    }
  }
}

fn gen_nonterm(
  nonterms: &mut Vec<Nonterm>,
  prods: &mut Vec<Production>,
  symbols: &mut HashMap<String, Symbol>,
  name: impl Into<String>,
  rule: RuleInner,
) -> NontermId {
  match rule {
    RuleInner::Sym(_) | RuleInner::Seq(_) => {
      let name = name.into();
      let prod = gen_prod(nonterms, prods, symbols, ProdAction::None, rule);
      let prod_ix = prods.len();
      prods.push(prod);
      let nonterm = Nonterm {
        name: name.clone(),
        prod_range: prod_ix..prod_ix + 1,
      };
      insert_nonterm(nonterms, symbols, name, nonterm)
    }
    RuleInner::Or(rules) => {
      let name = name.into();
      let rule_prods = rules.into_iter()
        .map(|rule| gen_prod(nonterms, prods, symbols, ProdAction::None, rule))
        .collect::<Vec<_>>();
      let prod_range = prods.len()..prods.len() + rule_prods.len();
      prods.append(&mut rule_prods);
      let nonterm = Nonterm {
        name: name.clone(),
        prod_range,
      };
      insert_nonterm(nonterms, symbols, name, nonterm)
    }
    RuleInner::Many(box RuleRep { rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, |nonterms, symbols, id| {
        let sym = gen_sym(nonterms, symbols, rule);
        let prods = &mut nonterms[id.0 as usize].prods;

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
            sym,
          ],
          ..Default::default()
        });
      })
    }
    RuleInner::Many1(box RuleRep { rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, |nonterms, symbols, id| {
        let sym = gen_sym(nonterms, symbols, rule);
        let prods = &mut nonterms[id.0 as usize].prods;

        // rule+ -> rule
        prods.push(Production {
          action: ProdAction::StartMany1,
          symbols: vec![sym],
          ..Default::default()
        });

        // rule+ -> rule+ rule
        prods.push(Production {
          action: ProdAction::ContinueMany1,
          symbols: vec![
            Symbol::Nonterm(id),
            sym,
          ],
          ..Default::default()
        });
      })
    }
    RuleInner::Option(box RuleRep { rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, |nonterms, symbols, id| {
        let prod = gen_prod(nonterms, symbols, ProdAction::NonemptyOption, rule);
        let prods = &mut nonterms[id.0 as usize].prods;

        // rule? -> ε
        prods.push(Production {
          action: ProdAction::EmptyOption,
          ..Default::default()
        });

        // rule? -> rule
        prods.push(prod);
      })
    }
    RuleInner::SepBy(box RuleSepBy { sep, rule }) => {
      let name = name.into();
      let nonterm = Nonterm {
        name: name.clone(),
        prods: vec![
          // sepBy(sep, rule) -> ε
          Production {
            action: ProdAction::EmptySepBy,
            ..Default::default()
          },

          // sepBy(sep, rule) -> sepBy1(sep, rule)
          gen_prod(nonterms, symbols, ProdAction::NonemptySepBy,
            RuleInner::SepBy1(box RuleSepBy {
              sep,
              rule,
            }))
        ],
      };
      insert_nonterm(nonterms, symbols, name, nonterm)
    }
    RuleInner::SepBy1(box RuleSepBy { sep, rule }) => {
      gen_rep_nonterm(nonterms, symbols, id, name, |nonterms, symbols, id| {
        let sep_sym = gen_sym(nonterms, symbols, sep);
        let sym = gen_sym(nonterms, symbols, rule);
        let prods = &mut nonterms[id.0 as usize].prods;

        // sepBy1(sep, rule) -> rule
        prods.push(Production {
          action: ProdAction::StartSepBy1,
          symbols: vec![sym],
          ..Default::default()
        });

        // sepBy1(sep, rule) -> sepBy1(sep, rule) sep rule
        prods.push(Production {
          action: ProdAction::ContinueSepBy1,
          symbols: vec![
            Symbol::Nonterm(id),
            sep_sym,
            sym,
          ],
          ..Default::default()
        });
      })
    }
    RuleInner::Prec(box RulePrec { prec, assoc, rule }) => {
      let nonterm_id = gen_nonterm(nonterms, symbols, id, name, rule);
      for prod in &mut nonterms[nonterm_id.0 as usize].prods {
        prod.prec = Some(prec);
        prod.assoc = assoc;
      }
      nonterm_id
    }
  }
}

fn insert_nonterm(
  nonterms: &mut Vec<Nonterm>,
  symbols: &mut HashMap<String, Symbol>,
  name: String,
  nonterm: Nonterm,
) -> NontermId {
  if let Some(Symbol::Nonterm(id)) = symbols.get(&name) {
    nonterms[id.0 as usize] = nonterm;
    *id
  } else {
    let id = NontermId(nonterms.len() as u32);
    symbols.insert(name, Symbol::Nonterm(id));
    nonterms.push(nonterm);
    id
  }
}

fn gen_rep_nonterm<F>(
  nonterms: &mut Vec<Nonterm>,
  symbols: &mut HashMap<String, Symbol>,
  name: impl Into<String>,
  f: F,
) -> NontermId
where
  F: FnOnce(
    &mut Vec<Nonterm>,
    &mut HashMap<String, Symbol>,
    NontermId,
  )
{
  let name = name.into();
  let id = if let Some(id) = id {
    id
  } else {
    let id = NontermId(nonterms.len() as u32);
    nonterms.push(Nonterm::default());
    id
  };

  symbols.insert(name.clone(), Symbol::Nonterm(id));

  nonterms[id.0 as usize].name = name;
  f(nonterms, symbols, id);

  id
}

fn gen_prod(
  nonterms: &mut Vec<Nonterm>,
  prods: &mut Vec<Production>,
  symbols: &mut HashMap<String, Symbol>,
  action: ProdAction,
  rule: RuleInner,
) -> Production {
  match rule {
    RuleInner::Seq(rules) => {
      Production {
        action,
        symbols: rules.into_iter()
          .map(|rule| gen_sym(nonterms, symbols, rule))
          .collect(),
        ..Default::default()
      }
    }
    RuleInner::Prec(box RulePrec { prec, assoc, rule }) => {
      let mut prod = gen_prod(nonterms, symbols, action, rule);
      prod.prec = Some(prec);
      prod.assoc = assoc;
      prod
    }
    _ => {
      Production {
        action,
        symbols: vec![gen_sym(nonterms, symbols, rule)],
        ..Default::default()
      }
    }
  }
}

fn gen_sym(
  nonterms: &mut Vec<Nonterm>,
  symbols: &mut HashMap<String, Symbol>,
  rule: RuleInner,
) -> Symbol {
  match rule {
    RuleInner::Sym(sym) => {
      if !symbols.contains_key(&sym) {
        println!("symbols: {:#?}, sym: {}", symbols, sym)
      }
      symbols[&sym]
    },
    RuleInner::Prec(box RulePrec { prec, assoc, rule }) => {
      let id = match gen_sym(nonterms, symbols, rule.clone()) {
        Symbol::Term(_) => {
          gen_nonterm(nonterms, symbols, None, rule.name(), rule)
        }
        Symbol::Nonterm(id) => id,
      };
      for prod in &mut nonterms[id.0 as usize].prods {
        prod.prec = Some(prec);
        prod.assoc = assoc;
      }
      Symbol::Nonterm(id)
    }
    _ => {
      let name = rule.name();
      Symbol::Nonterm(gen_nonterm(nonterms, symbols, None, name, rule))
    }
  }
}

#[cfg(test)]
mod tests {
  use insta::assert_debug_snapshot;
  use super::*;

  #[test]
  fn expr() {
    let gram = grammar(
      &["+", "-", "*", "/", "num", "(", ")", "id", ","],
      &["expr"],
    &[
      (
        "expr",
        prec(0, Assoc::Left, seq([sym("expr"), sym("+") | sym("-"), sym("expr")]))
        | prec(1, Assoc::Left, seq([sym("expr"), sym("*") | sym("/"), sym("expr")]))
        | prec(2, Assoc::None, seq([sym("-"), sym("expr")]))
        | seq([sym("("), sym("expr"), sym(")")])
        | sym("id")
        | sym("num")
        | sym("call")
      ),
      (
        "call",
        seq([sym("id"), sym("("), sep_by(sym(","), sym("expr")), sym(")")])
      )
    ]).unwrap();

    gram.validate().unwrap();
    let bnf: Bnf = gram.into();

    assert_debug_snapshot!(bnf);
  }

  #[test]
  fn repetition() {
    let gram = grammar(
      &["a", "b", "c", "d"],
      &["A", "B"],
      &[
        (
          "A",
          seq([
            many(seq([sym("a"), option(sym("C")), sym("B") | sym("b")])),
            option(seq([sym("A"), sym("a")]))
          ])
        ),
        (
          "B",
          many1(sym("c") | seq([sym("d"), sym("B")]))
        ),
        (
          "C",
          sym("B") | sym("b")
        ),
      ]).unwrap();

    gram.validate().unwrap();
    let bnf: Bnf = gram.into();

    assert_debug_snapshot!(bnf);
  }
}