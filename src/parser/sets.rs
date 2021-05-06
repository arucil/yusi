use crate::bnf::*;

pub(super) fn gen_nullable(
  bnf: &Bnf,
) -> Vec<bool> {
  let mut nullable = vec![false; bnf.nonterms.len()];
  let mut changed = false;

  loop {
    for (i, nonterm) in bnf.nonterms.iter().enumerate() {
      for prod in &nonterm.prods {
        if prod.symbols.iter().all(|sym| is_nullable(&nullable, sym)) {
          changed |= nullable[i];
          nullable[i] = true;
        }
      }
    }
    if !changed {
      break;
    }
  }

  nullable
}

fn is_nullable(nullable: &[bool], sym: &Symbol) -> bool {
  match sym {
    Symbol::Term(_) => false,
    Symbol::Nonterm(id) => nullable[id.0 as usize],
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use indexmap::indexmap;

  /// ```bnf
  /// Z -> d | X Y Z
  /// Y -> ε | c
  /// X -> Y | a
  /// ```
  fn simple() -> Bnf {
    Bnf {
      tokens: indexmap! {
        "a".to_owned() => TermId(0),
        "c".to_owned() => TermId(1),
        "d".to_owned() => TermId(2),
      },
      start: indexmap! {
        "Z".to_owned() => NontermId(0),
        "Y".to_owned() => NontermId(1),
        "X".to_owned() => NontermId(2),
      },
      nonterms: vec![
        Nonterm {
          name: "Z".to_owned(),
          prods: vec![
            Production {
              symbols: vec![Symbol::Term(TermId(2))],
              ..Default::default()
            },
            Production {
              symbols: vec![
                Symbol::Nonterm(NontermId(2)),
                Symbol::Nonterm(NontermId(1)),
                Symbol::Nonterm(NontermId(0)),
              ],
              ..Default::default()
            },
          ]
        },
        Nonterm {
          name: "Y".to_owned(),
          prods: vec![
            Production {
              symbols: vec![],
              ..Default::default()
            },
            Production {
              symbols: vec![
                Symbol::Term(TermId(1)),
              ],
              ..Default::default()
            },
          ]
        },
        Nonterm {
          name: "X".to_owned(),
          prods: vec![
            Production {
              symbols: vec![Symbol::Nonterm(NontermId(1))],
              ..Default::default()
            },
            Production {
              symbols: vec![ Symbol::Term(TermId(0)) ],
              ..Default::default()
            },
          ]
        }
      ]
    }
  }

  #[test]
  fn simple_nullable() {
    let nullable = gen_nullable(&simple());
    assert_eq!(nullable, vec![false, true, true]);
  }
}