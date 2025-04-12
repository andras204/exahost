use super::TokenType;

#[derive(Debug, Clone)]
pub struct Signature(pub Vec<Vec<TokenType>>);

impl Signature {
    pub fn one(ttypes: &[TokenType]) -> Self {
        Self(vec![ttypes.to_owned(), vec![], vec![]])
    }

    pub fn two(ttypes1: &[TokenType], ttypes2: &[TokenType]) -> Self {
        Self(vec![ttypes1.to_owned(), ttypes2.to_owned(), vec![]])
    }

    pub fn three(ttypes1: &[TokenType], ttypes2: &[TokenType], ttypes3: &[TokenType]) -> Self {
        Self(vec![
            ttypes1.to_owned(),
            ttypes2.to_owned(),
            ttypes3.to_owned(),
        ])
    }

    pub fn label() -> Self {
        Self(vec![vec![TokenType::JumpLabel], vec![], vec![]])
    }

    pub fn r() -> Self {
        Self(vec![vec![TokenType::RegisterLabel], vec![], vec![]])
    }

    pub fn rn() -> Self {
        Self(vec![
            vec![TokenType::RegisterLabel, TokenType::Number],
            vec![],
            vec![],
        ])
    }

    pub fn math() -> Self {
        Self(vec![
            vec![TokenType::Number, TokenType::RegisterLabel],
            vec![TokenType::Number, TokenType::RegisterLabel],
            vec![TokenType::RegisterLabel],
        ])
    }

    pub fn empty() -> Self {
        Self(vec![vec![], vec![], vec![]])
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        if !self.0[0].is_empty() {
            len += 1;
        }
        if !self.0[1].is_empty() {
            len += 1;
        }
        if !self.0[2].is_empty() {
            len += 1;
        }
        len
    }
}
