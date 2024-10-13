pub(crate) mod key;
pub(crate) mod call;
pub(crate) mod list;
pub(crate) mod literal;
pub(crate) mod associative_array;
pub(crate) mod expression;
pub(crate) mod value;
pub(crate) mod test;

use crate::{error::*, parse::value::value_parser, parse::value::Value, Context, DataSource};
use alloc::{
    borrow::ToOwned,
    collections::BTreeMap,
    rc::Rc,
    vec,
    vec::Vec
};
use alloc::string::String;
use core::ops::Deref;

pub(super) mod parser {
    pub use nom::branch::*;
    pub use nom::bytes::complete::*;
    pub use nom::character::complete::*;
    pub use nom::combinator::*;
    pub use nom::multi::*;
    pub use nom::sequence::*;
}

pub(crate) mod objects {
    pub(crate) use crate::parse::value::Value;
    pub(crate) use crate::parse::literal::Literal;
    pub(crate) use crate::parse::expression::Expression;
    pub(crate) use crate::parse::call::Call;
    pub(crate) use crate::parse::key::Key;
}


struct ParseContext(Rc<ContextInner>);

impl ParseContext {
    fn new(precedences: Vec<i64>, operators: BTreeMap<i64, Vec<String>>) -> Self {
        Self(Rc::new(ContextInner {
            precedences,
            operators,
        }))
    }
}

impl Deref for ParseContext {
    type Target = ContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for ParseContext {
    fn clone(&self) -> Self {
        ParseContext(Rc::clone(&self.0))
    }
}

struct ContextInner {
    operators: BTreeMap<i64, Vec<String>>,
    precedences: Vec<i64>,
}

impl ParseContext {}

impl<Provider: DataSource> Context<Provider> {
    fn parse_context(&self) -> ParseContext {
        // Group the operators by precedence into a BTreeMap so it's sorted.
        let operators = self.operators.iter()
            .fold(BTreeMap::new(), |mut accumulator, (token, operator)| {
                if !accumulator.contains_key(&operator.precedence) {
                    accumulator.insert(operator.precedence, vec![]);
                }

                accumulator.get_mut(&operator.precedence).unwrap().push(token.clone());

                return accumulator;
            });

        ParseContext::new(
            operators.keys().copied().collect::<Vec<_>>(),
            operators,
        )
    }

    pub(crate) fn parse(&self, str: &str) -> Result<Value> {
        let parser = value_parser(self.parse_context());

        parser(str)
            .map(|(_, v)| v)
            .map_err(|err| stringify(err).into())
    }
}

fn stringify(err: nom::Err<nom::error::Error<&str>>) -> nom::Err<nom::error::Error<String>> {
    match err {
        nom::Err::Error(err) => nom::Err::Error(nom::error::Error {
            input: err.input.to_owned(),
            code: err.code,
        }),
        nom::Err::Failure(err) => nom::Err::Failure(nom::error::Error {
            input: err.input.to_owned(),
            code: err.code,
        }),
        nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
    }.into()
}