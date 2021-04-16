#![feature(box_syntax)]

pub mod grammar;
pub mod parser;

pub fn build(grammar: grammar::Grammar) {
  println!("Hello, world!");
}
