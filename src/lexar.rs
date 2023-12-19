use once_cell::sync::Lazy;
use regex::RegexSet;

#[derive(Debug, Clone)]
pub struct Token {
    pub token: String,
    pub token_type: TokenType,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenType {
    Instruction,
    Number,
    Keyword,
    Register,
    Label,
    Comparison,
    Comment,
}

impl Token {
    pub fn new(token: String, token_type: TokenType) -> Token {
        Token {
            token,
            token_type,
        }
    }

    pub fn instruction(&self) -> Result<String, &str> {
        match self.token_type {
            TokenType::Instruction => Ok(self.token.to_owned()),
            _ => Err("Not an instruction"),
        }
    }

    pub fn number(&self) -> Result<i16, &str> {
        match self.token_type {
            TokenType::Number => Ok(self.token.parse::<i16>().unwrap()),
            _ => Err("Not a number"),
        }
    }

    pub fn keyword(&self) -> Result<String, &str> {
        match self.token_type {
            TokenType::Keyword => {
                let mut a = self.token.clone();
                a.remove(0);
                a.remove(a.len() - 1);
                Ok(a)
            },
            _ => Err("Not a keyword"),
        }
    }

    pub fn register(&self) -> Result<char, &str> {
        match self.token_type {
            TokenType::Register => Ok(self.token.to_owned().pop().unwrap()),
            _ => Err("Not a register"),
        }
    }

    pub fn label(&self) -> Result<String, &str> {
        match self.token_type {
            TokenType::Label=> Ok(self.token.to_owned()),
            _ => Err("Not a label"),
        }
    }

    pub fn comparison(&self) -> Result<String, &str> {
        match self.token_type {
            TokenType::Comparison => Ok(self.token.to_owned()),
            _ => Err("Not a comaprison"),
        }
    }
}

pub fn compile(code: Vec<String>) -> Result<Vec<String>, Vec<String>> {
    let mut instructions: Vec<String> = Vec::new();
    let mut errs: Vec<String> = Vec::new();

    if code.len() == 0 {
        errs.push("Nothing to compile".to_string());
        return Err(errs);
    }

    for x in 0..code.len() {
        let tokens;
        match tokenize(code[x].clone()) {
            Ok(t) => tokens = t,
            Err(e) => {
                errs.push(format!("{} at line: {}", e, x + 1));
                continue;
            },
        }

        if tokens[0].token_type == TokenType::Comment { continue; }

        match sig_match(&tokens) {
            Ok(_) => instructions.push(code[x].clone()),
            Err(e) => errs.push(format!("{} at line: {}", e, x + 1)),
        }
    }

    if instructions.len() == 0 {
        errs.push("No instructions".to_string());
    }
    if errs.len() > 0 {
        return Err(errs);
    }
    Ok(instructions)
}

pub fn tokenize(instr: String) -> Result<Vec<Token>, &'static str> {
    let sliced = split_instruction(instr);
    let tokens: Vec<Token>;
    match parse_tokens(sliced) {
        Ok(t) => tokens = t,
        Err(e) => return Err(e),
    }
    Ok(tokens)
}

pub fn split_instruction(instr: String) -> Vec<String> {
    if instr.starts_with("note") || instr.starts_with(";") {
        return vec![ "note".to_string() ];
    }

    let mut sliced: Vec<String> = vec![ instr[..4].to_string() ];

    if instr.len() > 5 {
        let arg_slice = &instr[5..];
        let mut t_begin: usize = 0;
        let mut mid_word: bool = false;
        let mut x = 0;
        while x < arg_slice.len() {
            match arg_slice.chars().nth(x).unwrap() {
                '\'' => {
                    mid_word = !mid_word;
                    if !mid_word {
                        sliced.push(arg_slice[t_begin..(x + 1)].to_string());
                        t_begin = x + 1;
                        x += 1;
                    }
                },
                ' ' => {
                    if !mid_word {
                        sliced.push(arg_slice[t_begin..x].to_string());
                        t_begin = x + 1;
                    }
                },
                _ => {},
            }
            x += 1;
        }
        if arg_slice.len() > t_begin {
            sliced.push(arg_slice[t_begin..arg_slice.len()].to_string());
        }
    }
    sliced
}

fn parse_tokens(sliced: Vec<String>) -> Result<Vec<Token>, &'static str> {
    let mut tokens: Vec<Token>;

    if sliced[0] == "note" {
        tokens = vec![ Token::new(sliced[0].clone(), TokenType::Comment) ];
        return Ok(tokens);
    }

    tokens = vec![ Token::new(sliced[0].clone(), TokenType::Instruction) ];

    for x in 1..sliced.len() {
        let t_type: TokenType;
        match pattern_match(sliced[x].clone()) {
            Ok(t) => t_type = t,
            Err(e) => return Err(e),
        }
        tokens.push(Token::new(sliced[x].clone(), t_type));
    }
    Ok(tokens)
}

/// regex match to determine `TokenType`s
pub fn pattern_match(str: String) -> Result<TokenType, &'static str> {
    static RS: Lazy<RegexSet> = Lazy::new(|| RegexSet::new([
        r"'*'",          // Keyword
        r"[A-Z]+[0-9]*", // Label
        r"[0-9]+",       // Number 
        r"[xtfm]{1}",    // Register
        r"[=!><]{1,2}",  // Comparison
    ]).unwrap());
    match RS.matches(&str[..]).into_iter().nth(0).unwrap_or(9999) {
        0 => Ok(TokenType::Keyword),
        1 => Ok(TokenType::Label),
        2 => Ok(TokenType::Number),
        3 => Ok(TokenType::Register),
        4 => Ok(TokenType::Comparison),
        _ => Err("Unknown argument"),
    }
}

/// match `TokenType`s with Instruction signatures
/// to catch type mismatch errors before execution
fn sig_match(tokens: &Vec<Token>) -> Result<(), &str> {
    let sig: Vec<Vec<TokenType>>;
    let args = &tokens[1..];
    match get_instr_sig(&tokens[0]) {
        Ok(s) => sig = s,
        Err(e) => return Err(e),
    }
    if sig.len() != args.len() { return Err("Incorrect number of arguments"); }
    for x in 0..sig.len() {
        match match_arg_type(&sig[x], &args[x].token_type) {
            Ok(_) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn match_arg_type(sig: &Vec<TokenType>, arg_type: &TokenType) -> Result<(), &'static str> {
    for st in sig {
        if arg_type == st { return Ok(()); }
    }
    Err("Wrong argument type")
}

fn get_instr_sig(instr: &Token) -> Result< Vec<Vec<TokenType>>, &str > {
    match &instr.token[..] {
        "copy" => Ok(vec![
            vec![TokenType::Number, TokenType::Register],
            vec![TokenType::Register],
        ]),
        "addi" | "subi" | "muli" | "divi" | "modi" => Ok(vec![
            vec![TokenType::Number, TokenType::Register],
            vec![TokenType::Number, TokenType::Register],
            vec![TokenType::Register],
        ]),
        "test" => Ok(vec![
            vec![TokenType::Number, TokenType::Register],
            vec![TokenType::Comparison],
            vec![TokenType::Number, TokenType::Register],
        ]),
        "jump" | "tjmp" | "fjmp" | "mark" | "repl" => Ok(vec![
            vec![TokenType::Label],
        ]),
        "prnt" => Ok(vec![
            vec![TokenType::Number, TokenType::Register, TokenType::Keyword],
        ]),
        "link" => Ok(vec![
            vec![TokenType::Number, TokenType::Register],
        ]),
        "halt" | "kill" | "noop" => Ok(vec![]),
        _ => Err("Unknown Instruction"),
    }
}
