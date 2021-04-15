export const or = (...subrules: Rule[]): RuleOr => ({
  type: 'or',
  subrules,
})

export const many = (subrule: Rule): RuleMany => ({
  type: 'many',
  subrule,
})

export const some = (subrule: Rule): RuleSome => ({
  type: 'some',
  subrule,
})

export const option = (subrule: Rule): RuleOption => ({
  type: 'option',
  subrule,
})

export const sepBy = (sep: Rule, subrule: Rule): RuleSepBy => ({
  type: 'sepBy',
  sep,
  subrule,
})

export const sepBy1 = (sep: Rule, subrule: Rule): RuleSepBy1 => ({
  type: 'sepBy1',
  sep,
  subrule,
})

export const prec = (prec: number, subrule: Rule): RulePrec => ({
  type: 'prec',
  prec,
  subrule,
})

export type Grammar = {
  tokens: Sym[],
  rules: {
    [ruleName: string]: Rule
  }
}

export type RuleMany = {
  type: 'many',
  subrule: Rule,
}

export type RuleSome = {
  type: 'some',
  subrule: Rule,
}

export type RuleOption = {
  type: 'option',
  subrule: Rule,
}

export type RuleOr = {
  type: 'or',
  subrules: Rule[],
}

export type RuleSepBy = {
  type: 'sepBy',
  sep: Rule,
  subrule: Rule,
}

export type RuleSepBy1 = {
  type: 'sepBy1',
  sep: Rule,
  subrule: Rule,
}

export type RulePrec = {
  type: 'prec',
  prec: number,
  subrule: Rule,
}

export type Sym = string

export type Rule = Rule[] | Sym | RuleOr | RuleMany | RuleOption
  | RuleSepBy | RuleSepBy1 | RulePrec