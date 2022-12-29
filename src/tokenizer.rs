#[derive(Clone)]
enum Token {
    Ločilo(String),
    Ime(String),
}

struct Tokenizer {
    text: String,
    tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer { text: String::new(), tokens: Vec::new() }
    }

    pub fn from(text: String) -> Tokenizer {
        *Tokenizer::new().add_text(text)
    }

    pub fn add_text(&self, text: String) -> &Self {
        self.text = text.clone();
        self
    }

    pub fn tokenize(&mut self) {

    }

    pub fn as_slice(&self) -> &[Token] {
        self.tokenize();
        self.tokens.as_slice()
    }
}
