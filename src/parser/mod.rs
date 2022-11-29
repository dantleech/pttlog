pub mod timesheet;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Prose,
    Tag,
    Ticket,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub whitespace: String,
}

impl Token {
    pub fn tag(text: String) -> Token {
        Token {
            kind: TokenKind::Tag,
            text,
            whitespace: "".to_string(),
        }
    }
    pub fn to_string(&self) -> String {
        format!("{}{}", self.text.to_string(), self.whitespace.to_string())
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}
