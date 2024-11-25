use std::{fs, io::BufReader, path::PathBuf};

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::Register;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct File {
    content: Vec<Register>,
    ptr: i16,
}

impl File {
    pub fn read(&mut self) -> Option<Register> {
        let res = Some(self.content.get(self.ptr as usize)?.clone());
        self.ptr = i16::clamp(self.ptr + 1, 0, self.content.len() as i16);
        res
    }

    pub fn write(&mut self, value: Register) {
        if self.ptr >= self.content.len() as i16 {
            self.content.push(value);
        } else {
            self.content[self.ptr as usize] = value;
        }
        self.ptr = i16::clamp(self.ptr + 1, 0, self.content.len() as i16);
    }

    pub fn seek(&mut self, amount: i16) {
        self.ptr = i16::clamp(self.ptr + amount, 0, self.content.len() as i16);
    }

    pub fn is_eof(&self) -> bool {
        self.ptr == self.content.len() as i16
    }

    pub fn open(path: PathBuf) -> Self {
        let f = std::fs::File::open(path).unwrap();
        let reader = BufReader::new(f);
    }
}

impl Default for File {
    fn default() -> Self {
        Self::new("./hosts/swap")
    }
}

impl From<Vec<&str>> for File {
    fn from(value: Vec<&str>) -> Self {
        Self {
            path: "./hosts/swap".into(),
            content: value
                .into_iter()
                .map(|s| match s.parse::<i16>() {
                    Ok(n) => {
                        if !(-9999..=9999).contains(&n) {
                            Register::Keyword(n.to_string().into_boxed_str())
                        } else {
                            Register::Number(n)
                        }
                    }
                    Err(_) => Register::Keyword(s.into()),
                })
                .collect(),
            ptr: 0,
        }
    }
}
