use serde::{Deserialize, Serialize};

use crate::exa::Register;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    content: Vec<Register>,
    ptr: usize,
}

impl File {
    pub fn read(&mut self) -> Option<Register> {
        let res = Some(self.content.get(self.ptr)?.clone());
        self.ptr = usize::clamp(self.ptr + 1, 0, self.content.len());
        res
    }

    pub fn write(&mut self, value: Register) {
        if self.ptr >= self.content.len() {
            self.content.push(value);
        } else {
            self.content[self.ptr] = value;
        }
        self.ptr = usize::clamp(self.ptr + 1, 0, self.content.len());
    }

    pub fn seek(&mut self, amount: i16) {
        self.ptr = isize::clamp(
            self.ptr as isize + amount as isize,
            0,
            self.content.len() as isize,
        ) as usize;
    }

    pub fn is_eof(&self) -> bool {
        self.ptr == self.content.len()
    }

    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            ptr: 0,
        }
    }
}

impl Default for File {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<String>> for File {
    fn from(value: Vec<String>) -> Self {
        Self {
            content: value
                .into_iter()
                .map(|s| match s.parse::<i16>() {
                    Ok(n) => Register::Number(n),
                    Err(_) => Register::Keyword(s),
                })
                .collect(),
            ptr: 0,
        }
    }
}

impl From<Vec<&str>> for File {
    fn from(value: Vec<&str>) -> Self {
        Self {
            content: value
                .into_iter()
                .map(|s| match s.parse::<i16>() {
                    Ok(n) => Register::Number(n),
                    Err(_) => Register::Keyword(s.to_string()),
                })
                .collect(),
            ptr: 0,
        }
    }
}
