use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compiler::ArgType;

use crate::exa::Instruction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub extra_instructions: bool,
    pub keyword_literals: bool,
    pub full_comparisons: bool,
    // pub allow_multi_m: bool,
    pub keyword_delimiter: char,
    pub comment_prefixes: Vec<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self::custom(
            false,
            false,
            // false,
            false,
            '\'',
            vec!["note", ";;"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }
}

impl CompilerConfig {
    pub fn extended() -> Self {
        Self::custom(
            true,
            true,
            // true,
            true,
            '\'',
            vec!["note", ";;", "//", "#"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn custom(
        extra_instructions: bool,
        keyword_literals: bool,
        full_comparisons: bool,
        // allow_multi_m: bool,
        keyword_delimiter: char,
        comment_prefixes: Vec<String>,
    ) -> Self {
        Self {
            extra_instructions,
            keyword_literals,
            full_comparisons,
            // allow_multi_m,
            keyword_delimiter,
            comment_prefixes,
        }
    }

    pub fn generate_comparisons(&self) -> Vec<String> {
        match self.full_comparisons {
            true => vec!["=", ">", "<", ">=", "<=", "!="]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            false => vec!["=", ">", "<"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn generate_signatures(&self) -> HashMap<Instruction, Vec<Vec<ArgType>>> {
        let r = vec![ArgType::Register];
        let rn = vec![ArgType::Register, ArgType::Number];
        let vari = match self.keyword_literals {
            true => vec![ArgType::Register, ArgType::Number, ArgType::Keyword],
            false => rn.clone(),
        };
        let c = vec![ArgType::Comparison];
        let l = vec![ArgType::Label];

        let mut sigs: HashMap<Instruction, Vec<Vec<ArgType>>> = HashMap::from_iter([
            (Instruction::Copy, vec![vari.clone(), r.clone()]),
            (Instruction::Void, vec![r.clone()]),
            (Instruction::Addi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Subi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Muli, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Divi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Modi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Swiz, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Rand, vec![rn.clone(), rn.clone(), r.clone()]),
            (
                Instruction::Test,
                vec![vari.clone(), c.clone(), vari.clone()],
            ),
            (Instruction::TestEof, vec![]),
            (Instruction::TestMrd, vec![]),
            (Instruction::Jump, vec![l.clone()]),
            (Instruction::Fjmp, vec![l.clone()]),
            (Instruction::Tjmp, vec![l.clone()]),
            (Instruction::Make, vec![]),
            (Instruction::Grab, vec![rn.clone()]),
            (Instruction::File, vec![r.clone()]),
            (Instruction::Seek, vec![rn.clone()]),
            (Instruction::Mark, vec![l.clone()]),
            (Instruction::Drop, vec![]),
            (Instruction::Wipe, vec![]),
            (Instruction::Repl, vec![l.clone()]),
            (Instruction::Link, vec![rn.clone()]),
            (Instruction::Host, vec![r.clone()]),
            (Instruction::Noop, vec![]),
            (Instruction::Halt, vec![]),
            (Instruction::Kill, vec![]),
        ]);

        if self.extra_instructions {
            sigs.insert(Instruction::Prnt, vec![vari.clone()]);
        }

        sigs
    }
}
