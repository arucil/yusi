use crate::bnf::*;
use super::bitset::BitSet;

pub(super) fn gen_nullable(
  bnf: &Bnf,
) -> Vec<bool> {
  let mut nullable = vec![false; bnf.nonterms.len()];

  loop {
    let mut changed = false;
    for prod in &bnf.prods {
      if prod.symbols.iter().all(|sym| is_nullable(&nullable, sym)) {
        let nt_ix = prod.nonterm_id.0 as usize;
        changed |= nullable[nt_ix];
        nullable[nt_ix] = true;
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

pub(super) fn gen_first(
  bnf: &Bnf,
  nullable: &[bool],
) -> Vec<BitSet> {
  let mut buf = BitSet::new(bnf.tokens.len());
  let mut first = vec![buf.clone(); bnf.nonterms.len()];

  loop {
    let mut changed = false;
    for prod in &bnf.prods {
      buf.clear();
      compute_first_for_symbols(&mut buf, &first, nullable, &prod.symbols,
        None);
      changed |= first[prod.nonterm_id.0 as usize].union_with(&buf);
    }
    if !changed {
      break;
    }
  }

  first
}

pub(super) fn compute_first_for_symbols(
  result: &mut BitSet,
  first: &[BitSet],
  nullable: &[bool],
  symbols: &[Symbol],
  lookaheads: Option<&BitSet>,
) {
  for sym in symbols {
    match sym {
      Symbol::Term(id) => {
        result.insert(id.0 as usize);
        return;
      }
      Symbol::Nonterm(id) => {
        result.union_with(&first[id.0 as usize]);
        if !nullable[id.0 as usize] {
          return;
        }
      }
    }
  }

  if let Some(lookaheads) = lookaheads {
    result.union_with(lookaheads);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  fn simple() -> Bnf {
    Bnf::parse(r#"
      Z -> d
      Z -> X Y Z
      Y ->
      Y -> c
      X -> Y
      X -> a
    "#)
  }

  fn ll_expr() -> Bnf {
    Bnf::parse(r#"
      E -> T E'
      E' -> + T E'
      E' ->
      T -> F T'
      T' -> * F T'
      T' ->
      F -> num
      F -> ( E )
    "#)
  }

  #[test]
  fn simple_nullable() {
    let nullable = gen_nullable(&simple());
    assert_eq!(nullable, vec![false, true, true]);
  }

  #[test]
  fn ll_expr_nullable() {
    let nullable = gen_nullable(&ll_expr());
    assert_eq!(nullable, vec![false, true, false, true, false]);
  }

  #[test]
  fn simple_first() {
    let bnf = simple();
    let nullable = gen_nullable(&bnf);
    let first = gen_first(&bnf, &nullable)
      .into_iter()
      .map(|set|
        set.iter().map(|t| bnf.tokens.get_index(t).unwrap().0)
          .collect::<Vec<_>>())
      .collect::<Vec<_>>();
    assert_eq!(first,
      vec![
        vec!["d", "c", "a"],
        vec!["c"],
        vec!["c", "a"],
      ]);
  }

  #[test]
  fn ll_expr_first() {
    let bnf = ll_expr();
    let nullable = gen_nullable(&bnf);
    let first = gen_first(&bnf, &nullable)
      .into_iter()
      .map(|set|
        set.iter().map(|t| bnf.tokens.get_index(t).unwrap().0)
          .collect::<Vec<_>>())
      .collect::<Vec<_>>();
    assert_eq!(first,
      vec![
        vec!["num", "("],
        vec!["+"],
        vec!["num", "("],
        vec!["*"],
        vec!["num", "("]
      ]);
  }
}