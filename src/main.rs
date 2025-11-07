use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;
use std::io::Write;

/// Trait for a generic compiler.
/// Compilation, token management, and parsing.
pub trait Compiler {
    /// Begin compilation.
    fn compile(&mut self, source: &str);

    /// Advance to the next token and return it.
    fn next_token(&mut self) -> String;
    
    /// Start parsing the token stream.
    fn parse(&mut self);

    /// Get the current token without advancing.
    fn current_token(&self) -> String;

    /// Set the current token manually.
    fn set_current_token(&mut self, tok: String);
}

/// Trait for lexical analysis.
/// Tokenizes.
pub trait LexicalAnalyzer {
    /// Return the next character.
    fn get_char(&mut self) -> Option<char>;

    /// Add a character to the current lexeme.
    fn add_char(&mut self, c: char);

    /// Lookup if a string is a valid token.
    fn lookup(&self, s: &str) -> bool;

    /// Tokenize the entire source code into a vector of tokens.
    fn tokenize(&mut self, source: &str) -> Vec<String>;
}

/// Trait for syntax analysis and HTML generation.
pub trait SyntaxAnalyzer {
    /// Parse the entire document.
    fn parse_lolcode(&mut self);

    /// Parse the HEAD section of the document.
    fn parse_head(&mut self);

    /// Parse the TITLE element inside HEAD.
    fn parse_title(&mut self);

    /// Parse a comment block.
    fn parse_comment(&mut self);

    /// Parse the main body content.
    fn parse_body(&mut self);

    /// Parse a paragraph block.
    fn parse_paragraph(&mut self);

    /// Parse the inner content of a paragraph.
    fn parse_inner_paragraph(&mut self);

    /// Parse inline text or formatting inside a paragraph.
    fn parse_inner_text(&mut self);

    /// Parse a variable definition.
    fn parse_variable_define(&mut self);

    /// Parse a variable use.
    fn parse_variable_use(&mut self);

    /// Parse bold text.
    fn parse_bold(&mut self);

    /// Parse italics text.
    fn parse_italics(&mut self);

    /// Parse a list block.
    fn parse_list(&mut self);

    /// Parse items inside a list.
    fn parse_list_items(&mut self);

    /// Parse content inside a list item.
    fn parse_inner_list(&mut self);

    /// Parse an audio element.
    fn parse_audio(&mut self);

    /// Parse a video element.
    fn parse_video(&mut self);

    /// Parse a newline element.
    fn parse_newline(&mut self);

    /// Parse normal text content.
    fn parse_text(&mut self);

    /// Append content to HTML output.
    fn write_html(&mut self, content: &str);
}

/// Main compiler structure.
/// Stores tokens, variables, and generated HTML output.
pub struct LolCompiler {
    tokens: Vec<String>,
    current_index: usize,
    current: String,
    variables: HashMap<String, String>,
    html_output: String,
}

impl LolCompiler {
    /// Create a new instance of the compiler.
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            current_index: 0,
            current: String::new(),
            variables: HashMap::new(),
            html_output: String::new(),
        }
    }

    /// Print an error message and exit immediately.
    fn error(&self, msg: &str) -> ! {
        eprintln!("Syntax Error: {}", msg);
        process::exit(1);
    }

    /// Check that the current token matches the expected value.
    /// Advances to the next token if successful.
    fn match_token(&mut self, expected: &str) {
        if self.current_token().eq_ignore_ascii_case(expected) {
            self.next_token();
        } else {
            self.error(&format!("Expected '{}', found '{}'", expected, self.current_token()));
        }
    }
}

/// Implementation of the Compiler trait.
impl Compiler for LolCompiler {
    /// Begins the full compilation process for a string.
    /// This includes tokenizing, parsing, generating HTML.
    fn compile(&mut self, source: &str) {
        let mut lexer = LolLexer::new();
        self.tokens = lexer.tokenize(source);

        if self.tokens.is_empty() {
            self.error("Empty input file.");
        }

        self.current_index = 0;
        self.current = self.tokens[0].clone();
        self.parse();

        let filename = env::args().nth(1).unwrap();
        let html_filename = filename.replace(".lol", ".html");
        let mut file = fs::File::create(&html_filename).unwrap();
        file.write_all(self.html_output.as_bytes()).unwrap();
        println!("\nParsing completed successfully. HTML saved as {}", html_filename);

        if cfg!(target_os = "macos") {
            process::Command::new("open")
                .arg("-a")
                .arg("Google Chrome")
                .arg(&html_filename)
                .spawn()
                .ok();
        } else if cfg!(target_os = "windows") {
            process::Command::new("cmd")
                .args(&["/C", "start", "chrome", &html_filename])
                .spawn()
                .ok();
        }
    }

    /// Advances the compiler to the next token.
    fn next_token(&mut self) -> String {
        if self.current_index + 1 < self.tokens.len() {
            self.current_index += 1;
            self.current = self.tokens[self.current_index].clone();
        } else {
            self.current = String::new();
        }
        self.current.clone()
    }

    /// Invokes the main parsing routine for syntax analysis.
    fn parse(&mut self) {
        self.parse_lolcode();
    }

    /// Retrieves the current token being processed by the compiler.
    fn current_token(&self) -> String {
        self.current.clone()
    }

    /// Updates the current token with a new value.
    fn set_current_token(&mut self, tok: String) {
        self.current = tok;
    }
}

/// Lexer structure.
pub struct LolLexer {
    chars: Vec<char>,
    position: usize,
    current_lexeme: String,
}

impl LolLexer {
    /// Create a new instance of the lexer.
    pub fn new() -> Self {
        Self {
            chars: vec![],
            position: 0,
            current_lexeme: String::new(),
        }
    }

    /// Check if a character is whitespace.
    fn is_whitespace(c: char) -> bool {
        matches!(c, ' ' | '\t' | '\n' | '\r')
    }

    /// Check if a character is #.
    fn is_special(c: char) -> bool {
        c == '#'
    }
}

/// Implementation of the LexicalAnalyzer trait.
impl LexicalAnalyzer for LolLexer {
    /// Retrieves the next character from the input source.
    fn get_char(&mut self) -> Option<char> {
        if self.position < self.chars.len() {
            let ch = self.chars[self.position];
            self.position += 1;
            Some(ch)
        } else {
            None
        }
    }

    /// Adds a single character to the current lexeme being built.
    /// Used to turn characters into a complete token.
    fn add_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    /// Checks whether the provided string matches a valid keyword or symbol.
    fn lookup(&self, s: &str) -> bool {
        let valid = [
            "#HAI", "#KTHXBYE", "#OBTW", "#TLDR", "#MAEK", "#OIC", "#GIMMEH", "#MKAY", "#LEMME",
            "#I", "HAZ", "#IT", "IZ", "HEAD", "TITLE", "BOLD", "ITALICS", "NEWLINE", "SOUNDZ",
            "VIDZ", "PARAGRAF", "LIST", "ITEM", "SEE",
        ];
        valid.iter().any(|&v| v.eq_ignore_ascii_case(s))
    }

    /// Tokenizes the given source string into a vector of valid tokens.
    fn tokenize(&mut self, source: &str) -> Vec<String> {
        self.chars = source.chars().collect();
        self.position = 0;
        let mut tokens = vec![];

        while let Some(c) = self.get_char() {
            if Self::is_whitespace(c) {
                if !self.current_lexeme.is_empty() {
                    tokens.push(self.current_lexeme.clone());
                    self.current_lexeme.clear();
                }
                continue;
            }

            if Self::is_special(c) {
                if !self.current_lexeme.is_empty() {
                    tokens.push(self.current_lexeme.clone());
                    self.current_lexeme.clear();
                }

                let mut token = String::new();
                token.push(c);

                while let Some(next) = self.chars.get(self.position) {
                    if next.is_ascii_alphanumeric() || *next == ':' || *next == '/' || *next == '.' || *next == '?' || *next == '=' || *next == '&' || *next == '-' || *next == '%' {
                        token.push(*next);
                        self.position += 1;
                    } else {
                        break;
                    }
                }

                if !self.lookup(&token) && !token.starts_with("http") {
                    eprintln!("Lexical Error: Invalid token '{}'", token);
                    process::exit(1);
                }

                tokens.push(token);
            } else {
                self.add_char(c);
            }
        }

        if !self.current_lexeme.is_empty() {
            tokens.push(self.current_lexeme.clone());
        }

        tokens
    }
}

/// Implementation of the SyntaxAnalyzer trait.
impl SyntaxAnalyzer for LolCompiler {
    /// Parse the document.
    fn parse_lolcode(&mut self) {
        self.match_token("#HAI");
        while self.current_token().eq_ignore_ascii_case("#OBTW") {
            self.parse_comment();
        }
        self.parse_head();
        self.parse_body();
        self.match_token("#KTHXBYE");
    }

    /// Parse HEAD section.
    fn parse_head(&mut self) {
        if self.current_token().eq_ignore_ascii_case("#MAEK") {
            self.match_token("#MAEK");
            self.match_token("HEAD");
            while !self.current_token().eq_ignore_ascii_case("#OIC") {
                if self.current_token().eq_ignore_ascii_case("#GIMMEH") {
                    self.parse_title();
                } else {
                    self.next_token();
                }
            }
            self.match_token("#OIC");
        }
    }

    /// Parse TITLE element in HEAD.
    fn parse_title(&mut self) {
        self.match_token("#GIMMEH");
        self.match_token("TITLE");
        let mut title_text = String::new();
        while !self.current_token().eq_ignore_ascii_case("#MKAY") {
            title_text.push_str(&self.current_token());
            title_text.push(' ');
            self.next_token();
        }
        self.match_token("#MKAY");
        println!("Parsed Title: {}", title_text.trim());
        self.html_output.push_str(&format!("<html><head><title>{}</title></head><body>\n", title_text.trim()));
    }

    /// Parse a comment block.
    fn parse_comment(&mut self) {
        self.match_token("#OBTW");
        let mut comment_text = String::new();
        while !self.current_token().eq_ignore_ascii_case("#TLDR") {
            comment_text.push_str(&self.current_token());
            comment_text.push(' ');
            self.next_token();
        }
        self.match_token("#TLDR");
        println!("Comment: {}", comment_text.trim());
    }

    /// Parse main body content.
    fn parse_body(&mut self) {
        while !self.current_token().eq_ignore_ascii_case("#KTHXBYE") && !self.current_token().is_empty() {
            match self.current_token().to_uppercase().as_str() {
                "#MAEK" => self.parse_paragraph(),
                "#GIMMEH" => self.parse_inner_text(),
                "#LEMME" => self.parse_variable_use(),
                "#I" => self.parse_variable_define(),
                _ => self.parse_text(),
            }
        }
        self.html_output.push_str("</body></html>");
        println!();
    }

    /// Parse a paragraph block.
    fn parse_paragraph(&mut self) {
        self.match_token("#MAEK");
        self.match_token("PARAGRAF");
        self.html_output.push_str("<p>");
        self.parse_inner_paragraph();
        self.html_output.push_str("</p>\n");
        self.match_token("#OIC");
    }

    /// Parse inner content of a paragraph.
    fn parse_inner_paragraph(&mut self) {
        while !self.current_token().eq_ignore_ascii_case("#OIC") && !self.current_token().is_empty() {
            self.parse_inner_text();
        }
    }

    /// Parse inline text, formatting, variables
    fn parse_inner_text(&mut self) {
        match self.current_token().to_uppercase().as_str() {
            "#GIMMEH" => {
                self.next_token();
                match self.current_token().to_uppercase().as_str() {
                    "BOLD" => self.parse_bold(),
                    "ITALICS" => self.parse_italics(),
                    "NEWLINE" => self.parse_newline(),
                    "SOUNDZ" => self.parse_audio(),
                    "VIDZ" => self.parse_video(),
                    _ => self.error("Unknown GIMMEH construct"),
                }
            }
            "#LEMME" => self.parse_variable_use(),
            "#I" => self.parse_variable_define(),
            _ => self.parse_text(),
        }
    }

    /// Parse a variable definition.
    fn parse_variable_define(&mut self) {
        self.match_token("#I");
        self.match_token("HAZ");
        let var_name = self.current_token();
        self.next_token();
        self.match_token("#IT");
        self.match_token("IZ");
        let mut var_value = String::new();
        while !self.current_token().eq_ignore_ascii_case("#MKAY") {
            var_value.push_str(&self.current_token());
            var_value.push(' ');
            self.next_token();
        }
        self.variables.insert(var_name.clone(), var_value.trim().to_string());
        self.match_token("#MKAY");
    }

    /// Parse a variable usage.
    fn parse_variable_use(&mut self) {
        self.match_token("#LEMME");
        self.match_token("SEE");
        let var_name = self.current_token();
        self.next_token();
        self.match_token("#MKAY");

        if let Some(val) = self.variables.get(&var_name) {
            print!("{}", val);
            self.html_output.push_str(&val);
        } else {
            self.error(&format!("Undefined variable '{}'", var_name));
        }
        print!(" ");
        self.html_output.push_str(" ");
    }

    /// Parse bold text.
    fn parse_bold(&mut self) {
        self.match_token("BOLD");
        self.html_output.push_str("<b>");
        while !self.current_token().eq_ignore_ascii_case("#MKAY") && !self.current_token().is_empty() {
            print!("{} ", self.current_token());
            self.html_output.push_str(&format!("{} ", self.current_token()));
            self.next_token();
        }
        self.match_token("#MKAY");
        self.html_output.push_str("</b>");
    }

    /// Parse italics text.
    fn parse_italics(&mut self) {
        self.match_token("ITALICS");
        self.html_output.push_str("<i>");
        while !self.current_token().eq_ignore_ascii_case("#MKAY") && !self.current_token().is_empty() {
            print!("{} ", self.current_token());
            self.html_output.push_str(&format!("{} ", self.current_token()));
            self.next_token();
        }
        self.match_token("#MKAY");
        self.html_output.push_str("</i>");
    }

    /// Parse a list block.
    fn parse_list(&mut self) {
        self.match_token("#MAEK");
        self.match_token("LIST");
        self.html_output.push_str("<ul>\n");
        self.parse_list_items();
        self.html_output.push_str("</ul>\n");
        self.match_token("#OIC");
    }

    /// Parse items inside a list.
    fn parse_list_items(&mut self) {
        while self.current_token().eq_ignore_ascii_case("#GIMMEH") {
            self.match_token("#GIMMEH");
            self.match_token("ITEM");
            self.html_output.push_str("<li>");
            self.parse_inner_list();
            self.html_output.push_str("</li>\n");
            self.match_token("#MKAY");
        }
    }

    /// Parse inner content inside a list item.
    fn parse_inner_list(&mut self) {
        self.parse_inner_text();
    }

    /// Parse audio element.
    fn parse_audio(&mut self) {
        self.match_token("SOUNDZ");
        let audio_address = self.current_token();
        self.next_token();
        self.match_token("#MKAY");

        self.html_output.push_str(&format!(
            "<audio controls>\n  <source src=\"{}\" type=\"audio/mpeg\">\n</audio>\n",
            &audio_address
        ));

        println!("Audio URL: {}", audio_address);
    }

    /// Parse video element.
    fn parse_video(&mut self) {
        self.match_token("VIDZ");
        let video_address = self.current_token();
        self.next_token();
        self.match_token("#MKAY");

        self.html_output.push_str(&format!(
            "<video controls>\n  <source src=\"{}\" type=\"video/mp4\">\n</video>\n",
            &video_address
        ));

        println!("Video URL: {}", video_address);
    }

    /// Parse newline element.
    fn parse_newline(&mut self) {
        self.match_token("NEWLINE");
        print!("\n");
        self.html_output.push_str("<br/>\n");
    }

    /// Parse standard text.
    fn parse_text(&mut self) {
        let token = self.current_token();
        if !token.starts_with('#') && token.chars().all(|c| c.is_ascii()) {
            if [".", ",", "!", "?"].contains(&token.as_str()) {
                print!("{}", token);
                self.html_output.push_str(&token);
            } else {
                print!("{} ", token);
                self.html_output.push_str(&format!("{} ", &token));
            }
            self.next_token();
        } else {
            self.error(&format!("Expected text, found '{}'", token));
        }
    }

    /// Append content to HTML output.
    fn write_html(&mut self, content: &str) {
        self.html_output.push_str(content);
    }
}

/// Main execution for the compiler.
/// Reads file, verifies `.lol` extension, and compiles.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lolcompiler <input.lol>");
        process::exit(1);
    }

    let filename = &args[1];

    if !filename.to_lowercase().ends_with(".lol") {
        eprintln!("Error: Only .lol files are accepted. '{}' is invalid.", filename);
        process::exit(1);
    }

    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Error: could not read file '{}'", filename);
        process::exit(1);
    });

    let mut compiler = LolCompiler::new();
    compiler.compile(&contents);
}