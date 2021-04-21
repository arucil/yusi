use std::collections::HashSet;
use indexmap::IndexMap;
use std::ops::BitOr;

pub struct Grammar {
  pub(crate) tokens: Vec<String>,
  pub(crate) start: Vec<String>,
  pub(crate) rules: IndexMap<String, Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule(pub(crate) RuleInner);

#[derive(Debug, Clone)]
pub(crate) enum RuleInner {
  Sym(String),
  Seq(Vec<RuleInner>),
  Or(Vec<RuleInner>),
  Many(Box<RuleRep>),
  Some(Box<RuleRep>),
  Option(Box<RuleRep>),
  SepBy(Box<RuleSepBy>),
  SepBy1(Box<RuleSepBy>),
  Prec(Box<RulePrec>),
}

#[derive(Debug, Clone)]
pub(crate) struct RuleRep {
  pub(crate) rule: RuleInner,
}

#[derive(Debug, Clone)]
pub(crate) struct RuleSepBy {
  pub(crate) sep: RuleInner,
  pub(crate) rule: RuleInner,
}

#[derive(Debug, Clone)]
pub(crate) struct RulePrec {
  pub(crate) prec: u16,
  pub(crate) assoc: Assoc,
  pub(crate) rule: RuleInner,
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
  Rule(RuleInner::Sym(sym.into()))
}

pub fn seq<const N: usize>(
  rules: [Rule; N],
) -> Rule {
  Rule(RuleInner::Seq(rules.iter().map(|r| r.0.clone()).collect()))
}

pub fn many(
  rule: Rule,
) -> Rule {
  Rule(RuleInner::Many(box RuleRep {
    rule: rule.0,
  }))
}

pub fn some(
  rule: Rule,
) -> Rule {
  Rule(RuleInner::Some(box RuleRep {
    rule: rule.0,
  }))
}

pub fn option(
  rule: Rule,
) -> Rule {
  Rule(RuleInner::Option(box RuleRep {
    rule: rule.0,
  }))
}

pub fn sep_by(
  sep: Rule,
  rule: Rule,
) -> Rule {
  Rule(RuleInner::SepBy(box RuleSepBy {
    sep: sep.0,
    rule: rule.0,
  }))
}

pub fn sep_by1(
  sep: Rule,
  rule: Rule,
) -> Rule {
  Rule(RuleInner::SepBy1(box RuleSepBy {
    sep: sep.0,
    rule: rule.0,
  }))
}

pub fn prec(
  prec: u16,
  assoc: Assoc,
  rule: Rule,
) -> Rule {
  Rule(RuleInner::Prec(box RulePrec {
    prec,
    assoc,
    rule: rule.0,
  }))
}

impl BitOr for Rule {
  type Output = Rule;

  fn bitor(self, rhs: Rule) -> Rule {
    match (self.0, rhs.0) {
      (RuleInner::Or(mut x), RuleInner::Or(mut y)) => {
        x.append(&mut y);
        Rule(RuleInner::Or(x))
      }
      (RuleInner::Or(mut x), y) => {
        x.push(y);
        Rule(RuleInner::Or(x))
      }
      (x, RuleInner::Or(mut y)) => {
        y.insert(0, x);
        Rule(RuleInner::Or(y))
      }
      (x, y) => {
        Rule(RuleInner::Or(vec![x, y]))
      }
    }
  }
}

pub fn grammar(
  tokens: &[&str],
  start: &[&str],
  rules: &[(&str, Rule)],
) -> Result<Grammar, String> {
  let rules_map = rules.iter()
    .map(|(name, rule)| ((*name).to_owned(), rule.clone()))
    .collect::<IndexMap<_, _>>();
  if rules_map.len() != rules.len() {
    return Err(format!("duplicate rule found in rule list"));
  }
  
  Ok(Grammar {
    tokens: tokens.iter().map(|&s| s.to_owned()).collect(),
    start: start.iter().map(|&s| s.to_owned()).collect(),
    rules: rules_map,
  })
}

impl Grammar {
  pub(crate) fn validate(&self) -> Result<(), String> {
    if self.tokens.is_empty() {
      return Err(format!("token list is empty"));
    }

    let token_names = self.tokens.iter()
      .map(|k| k.as_str())
      .collect::<HashSet<_>>();

    if token_names.len() != self.tokens.len() {
      return Err(format!("duplicate token found in token list"))
    }

    if self.start.is_empty() {
      return Err(format!("start rule list is empty"))
    }

    if self.start.iter().collect::<HashSet<_>>().len() < self.start.len() {
      return Err(format!("duplicate token found in start list"))
    }

    let rule_names = self.rules.iter()
      .map(|(k, _)| k.as_str())
      .collect::<HashSet<_>>();

    for start in &self.start {
      if !rule_names.contains(start.as_str()) {
        return Err(format!("start rule '{}' is undefined", start))
      }
    }

    let names = token_names.union(&rule_names)
      .map(|&s| s)
      .collect::<HashSet<_>>();

    if names.len() != token_names.len() + rule_names.len() {
      return Err(format!("token name collides with rule name"));
    }

    for (_, rule) in &self.rules {
      rule.0.validate(&names)?;
    }

    Ok(())
  }
}

impl RuleInner {
  fn validate(&self, names: &HashSet<&str>) -> Result<(), String> {
    match self {
      Self::Sym(sym) => {
        if !names.contains(sym.as_str()) {
          Err(format!("symbol '{}' is undefined", sym))
        } else {
          Ok(())
        }
      }
      Self::Seq(rules) | Self::Or(rules) => {
        for rule in rules {
          rule.validate(names)?;
        }
        Ok(())
      }
      Self::Many(rule) | Self::Some(rule) | Self::Option(rule) => {
        rule.rule.validate(names)
      }
      Self::SepBy(rule) | Self::SepBy1(rule) => {
        rule.sep.validate(names)?;
        rule.rule.validate(names)
      }
      Self::Prec(rule) => {
        rule.rule.validate(names)
      }
    }
  }
}

impl Default for Assoc {
  fn default() -> Self {
    Self::None
  }
}