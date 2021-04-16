use std::collections::HashMap;
use std::ops::BitOr;

pub struct Grammar {
  tokens: Vec<String>,
  start: Vec<String>,
  rules: HashMap<String, Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule(RuleVariant);

#[derive(Debug, Clone)]
enum RuleVariant {
  Sym(String),
  Seq(Vec<Rule>),
  Or(Vec<Rule>),
  Many(Box<RuleRep>),
  Some(Box<RuleRep>),
  Option(Box<RuleRep>),
  SepBy(Box<RuleSepBy>),
  SepBy1(Box<RuleSepBy>),
  Prec(Box<RulePrec>),
}

#[derive(Debug, Clone)]
struct RuleRep {
  rule: Rule,
}

#[derive(Debug, Clone)]
struct RuleSepBy {
  sep: Rule,
  rule: Rule,
}

#[derive(Debug, Clone)]
struct RulePrec {
  prec: u16,
  assoc: Assoc,
  rule: Rule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assoc {
  None,
  Left,
  Right,
}

pub fn sym(
  sym: impl Into<String>,
) -> Rule {
  Rule(RuleVariant::Sym(sym.into()))
}

pub fn seq<const N: usize>(
  rules: [Rule; N],
) -> Rule {
  Rule(RuleVariant::Seq(rules.to_vec()))
}

pub fn many(
  rule: Rule,
) -> Rule {
  Rule(RuleVariant::Many(box RuleRep {
    rule,
  }))
}

pub fn some(
  rule: Rule,
) -> Rule {
  Rule(RuleVariant::Some(box RuleRep {
    rule,
  }))
}

pub fn option(
  rule: Rule,
) -> Rule {
  Rule(RuleVariant::Option(box RuleRep {
    rule,
  }))
}

pub fn sepBy(
  sep: Rule,
  rule: Rule,
) -> Rule {
  Rule(RuleVariant::SepBy(box RuleSepBy {
    sep,
    rule,
  }))
}

pub fn sepBy1(
  sep: Rule,
  rule: Rule,
) -> Rule {
  Rule(RuleVariant::SepBy1(box RuleSepBy {
    sep,
    rule,
  }))
}

pub fn prec(
  prec: u16,
  assoc: Assoc,
  rule: Rule,
) -> Rule {
  Rule(RuleVariant::Prec(box RulePrec {
    prec,
    assoc,
    rule,
  }))
}

impl BitOr for Rule {
  type Output = Rule;

  fn bitor(self, rhs: Rule) -> Rule {
    match (self.0, rhs.0) {
      (RuleVariant::Or(mut x), RuleVariant::Or(mut y)) => {
        x.append(&mut y);
        Rule(RuleVariant::Or(x))
      }
      (RuleVariant::Or(mut x), y) => {
        x.push(Rule(y));
        Rule(RuleVariant::Or(x))
      }
      (x, RuleVariant::Or(mut y)) => {
        y.insert(0, Rule(x));
        Rule(RuleVariant::Or(y))
      }
      (x, y) => {
        Rule(RuleVariant::Or(vec![Rule(x), Rule(y)]))
      }
    }
  }
}

pub fn grammar(
  tokens: &[&str],
  start: &[&str],
  rules: &[(&str, Rule)],
) -> Grammar {
  Grammar {
    tokens: tokens.iter().map(|&s| s.to_owned()).collect(),
    start: start.iter().map(|&s| s.to_owned()).collect(),
    rules: rules.iter()
      .map(|(name, rule)| ((*name).to_owned(), rule.clone()))
      .collect(),
  }
}