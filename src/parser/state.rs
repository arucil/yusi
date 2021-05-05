use std::collections::HashMap;
use crate::bnf::*;
use indexmap::IndexMap;
use super::token_set::TokenSet;

pub(super) struct States {
  pub(super) states: IndexMap<StateKey, State>,
  /// start symbol -> index of starting state
  pub(super) starts: HashMap<NontermId, u32>,
}

pub(crate) struct Lr0Item {
  pub(crate) nonterm: NontermId,
  pub(crate) prod_ix: u16,
}

pub(crate) struct Lalr1Item {
  pub(crate) nonterm: NontermId,
  pub(crate) prod_ix: u16,
  pub(crate) lookaheads: TokenSet,
}

pub(crate) struct State {
  pub(crate) items: Vec<Lalr1Item>,
  pub(crate) kernel_len: u16,
  /// symbol -> index of target state
  pub(crate) transitions: IndexMap<Symbol, u32>,
}

/// LALR(1) kernel items
type StateKey = Vec<Lr0Item>;

pub(super) fn gen_states(
  bnf: &Bnf,
) -> States {
  let mut states = IndexMap::new();
  let mut starts = HashMap::new();

  for &start in bnf.start.values() {
    let start_state = gen_states_for_start(bnf, &mut states, start);
    starts.insert(start, start_state);
  }

  States {
    states,
    starts,
  }
}

/// Generates states for a start symbol.
fn gen_states_for_start(
  bnf: &Bnf,
  states: &mut IndexMap<StateKey, State>,
  start: NontermId,
) -> u32 {
}