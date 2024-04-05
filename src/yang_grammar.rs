use crate::yang_grammar_trait::{Yang, YangGrammarTrait};
#[allow(unused_imports)]
use parol_runtime::{Result, Token};
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our Yang grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct YangGrammar<'t> {
    pub yang: Option<Yang<'t>>,
}

impl YangGrammar<'_> {
    pub fn new() -> Self {
        YangGrammar::default()
    }
}

impl Display for Yang<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for YangGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.yang {
            Some(yang) => writeln!(f, "{}", yang),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> YangGrammarTrait<'t> for YangGrammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal 'Yang'
    fn yang(&mut self, arg: &Yang<'t>) -> Result<()> {
        self.yang = Some(arg.clone());
        Ok(())
    }
}
