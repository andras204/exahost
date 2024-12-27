use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compiler::{Signature, TokenType};

use crate::instruction::OpCode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub extra_instructions: bool,
    pub keyword_literals: bool,
    pub full_comparisons: bool,
    pub keyword_delimiter: char,
    pub comment_prefixes: Vec<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self::custom(
            false,
            false,
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
        keyword_delimiter: char,
        comment_prefixes: Vec<String>,
    ) -> Self {
        Self {
            extra_instructions,
            keyword_literals,
            full_comparisons,
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

    pub fn generate_signatures(&self) -> HashMap<OpCode, Signature> {
        let r = vec![TokenType::RegisterLabel];
        let rn = vec![TokenType::RegisterLabel, TokenType::Number];
        let vari = match self.keyword_literals {
            true => vec![
                TokenType::RegisterLabel,
                TokenType::Number,
                TokenType::Keyword,
            ],
            false => vec![TokenType::RegisterLabel, TokenType::Number],
        };
        let c = vec![TokenType::Comparison];
        let l = vec![TokenType::JumpLabel];

        let mut sigs: HashMap<OpCode, Signature> = HashMap::from_iter([
            (OpCode::Copy, Signature::two(&vari, &r)),
            (OpCode::Void, Signature::r()),
            (OpCode::Addi, Signature::math()),
            (OpCode::Subi, Signature::math()),
            (OpCode::Muli, Signature::math()),
            (OpCode::Divi, Signature::math()),
            (OpCode::Modi, Signature::math()),
            (OpCode::Swiz, Signature::math()),
            (OpCode::Rand, Signature::math()),
            (OpCode::Test, Signature::three(&vari, &c, &vari)),
            (OpCode::TestEof, Signature::empty()),
            (OpCode::TestMrd, Signature::empty()),
            (OpCode::Jump, Signature::label()),
            (OpCode::Fjmp, Signature::label()),
            (OpCode::Tjmp, Signature::label()),
            (OpCode::Make, Signature::empty()),
            (OpCode::Grab, Signature::rn()),
            (OpCode::File, Signature::r()),
            (OpCode::Seek, Signature::rn()),
            (OpCode::Mark, Signature::label()),
            (OpCode::Drop, Signature::empty()),
            (OpCode::Wipe, Signature::empty()),
            (OpCode::Repl, Signature::label()),
            (OpCode::Link, Signature::rn()),
            (OpCode::Host, Signature::r()),
            (OpCode::Noop, Signature::empty()),
            (OpCode::Halt, Signature::empty()),
            (OpCode::Kill, Signature::empty()),
        ]);

        // if self.extra_instructions {
        //     sigs.insert(OpCode::Prnt, Signature::one(&vari));
        // }

        sigs
    }
}
