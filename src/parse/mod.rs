mod key;
mod call;
mod list;
mod literal;
mod associative_array;
mod expression;
mod value;
mod test;

use crate::{
    error::*,
    parse::value::value_parser,
    parse::value::Value,
    parse::value::OPERATORS,
};
use std::{
    collections::BTreeMap,
    ops::Deref,
    rc::Rc,
};

pub(super) mod parser {
    pub use nom::branch::*;
    pub use nom::bytes::complete::*;
    pub use nom::character::complete::*;
    pub use nom::combinator::*;
    pub use nom::multi::*;
    pub use nom::sequence::*;
}

struct Context(Rc<ContextInner>);

impl Context {
    fn new(precedences: Vec<i64>, operators: BTreeMap<i64, Vec<&'static str>>) -> Self {
        Self(Rc::new(ContextInner {
            precedences,
            operators,
        }))
    }
}

impl Deref for Context {
    type Target = ContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Context(Rc::clone(&self.0))
    }
}

struct ContextInner {
    operators: BTreeMap<i64, Vec<&'static str>>,
    precedences: Vec<i64>,
}

impl Context {}

pub fn parse(str: &str) -> Result<Value> {
    // Group the operators by precedence into a BTreeMap so it's sorted.
    let operators = OPERATORS.iter()
        .fold(BTreeMap::new(), |mut accumulator, (token, precedence, _num_operands)| {
            if !accumulator.contains_key(precedence) {
                accumulator.insert(*precedence, vec![]);
            }

            accumulator.get_mut(precedence).unwrap().push(*token);

            return accumulator;
        });

    let parser = value_parser(Context::new(
        operators.keys().copied().collect::<Vec<_>>(),
        operators,
    ));

    parser(str)
        .map(|(_, v)| v)
        .map_err(|err| match err {
            nom::Err::Error(err) => nom::Err::Error(nom::error::Error {
                input: err.input.to_owned(),
                code: err.code,
            }),
            nom::Err::Failure(err) => nom::Err::Failure(nom::error::Error {
                input: err.input.to_owned(),
                code: err.code,
            }),
            nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
        }.into())
}