use std::io;
use std::io::{Read, Cursor};
use std::process;

struct Compiler {
    // The next character we're going to consider.
    lookahead: Option<char>,

    // Our input stream of bytes. Cursor is so dope.
    input_stream: Cursor<Vec<u8>>,
}

impl Compiler {
    fn new(program: Vec<u8>) -> Compiler {
        Compiler {
            lookahead: None,
            input_stream: Cursor::new(program),
        }
    }

    fn init(&mut self) {
        self.get_char();
        self.skip_whitespace();
    }

    fn emit(&mut self) {
        self.init();
        self.emit_module_start();
        self.emit_main_start();
        
        if let Some(c) = self.lookahead {
            if c != '\n' {
                self.expected("newline", &c.to_string());
            }
        } else {
            self.expected("newline", "nothing");
        }
        
        self.emit_main_end();
        self.emit_module_end();
    }

    /// Consumes the next byte in the stream, converts it to a character,
    /// stores it in the lookahead, and returns the character.
    fn get_char(&mut self) -> Option<char> {
        // Read a single byte from the stream.
        let mut buf = [0];
        let result = self.input_stream.read_exact(&mut buf).ok();
        let byte = match result {
            Some(_) => buf[0],
            None => {
                self.lookahead = None;
                return None;
            }
        };

        // Convert the byte into an ASCII character.
        self.lookahead = Some(char::from(byte));
        self.lookahead
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(lookahead) = self.lookahead {
                if lookahead == ' ' {
                    self.get_char();
                } else {
                    break;
                }
        } else {
                break;
            }
        }
    }

    /// Prints an error message.
    fn error(&self, msg: &str) {
        eprintln!("Error: {}", msg);
    }
    
    /// Prints an error message and exits.
    fn abort(&self, msg: &str) {
        self.error(msg);
        process::exit(1);
    }

    /// Prints an error message prefixed with "expected" and exits.
    fn expected(&self, what: &str, found: &str) {
        self.abort(&format!("expected {}, found {}", what, found));
    }

    /// If the current lookahead is not equal to the matching character,
    /// prints an error and exits. Otherwise, consumes another character from
    /// the input stream, puts it in the lookahead, and returns it.
    fn consume_exact_char(&mut self, c: char) -> Option<char> {
        if let Some(lookahead) = self.lookahead {
            if lookahead == c {
                self.get_char();
                self.skip_whitespace();
                Some(lookahead)
            } else {
                self.expected(&c.to_string(), &lookahead.to_string());
                None
            }
        } else {
            self.expected(&c.to_string(), "nothing");
            None
        }
    }

    fn consume_name(&mut self) -> Option<String> {
        let mut name = String::from("");
        
        loop {
            if let Some(lookahead) = self.lookahead {
                if lookahead.is_ascii_alphanumeric() {
                    name.push(lookahead);
                    self.get_char();
                } else {
                    break;
                }
            } else {
                self.expected("name", "nothing");
            }
        }

        self.skip_whitespace();
        Some(name)
    }

    /// If the current lookahead is not a digit, prints an error and exits.
    /// Otherwise, consumes another byte from the input stream, puts it in the
    /// lookahead, and returns it.
    fn consume_num(&mut self) -> Option<String> {
        let mut num = String::from("");
        
        loop {
            if let Some(lookahead) = self.lookahead {
                if lookahead.is_digit(10) {
                    num.push(lookahead);
                    self.get_char();
                } else {
                    break;
                }
            } else {
                self.expected("integer", "nothing");
            }
        }

        self.skip_whitespace();
        Some(num)
    }
    
    /// Prints the start of a new WebAssembly module.
    fn emit_module_start(&self) {
        println!("(module");
    }

    /// Prints the closing paren of a WebAssembly module.
    fn emit_module_end(&self) {
        println!(")");
    }

    /// Prints the start of a function called main.
    fn emit_main_start(&self) {
        println!("(func $main (result i32)");
    }

    /// Prints the closing paren and export statement of the main function.
    fn emit_main_end(&self) {
        println!("(return)");
        println!(")");
        println!("(export \"main\" (func $main))");
    }
}

fn main() {
    // I slurp up everything from stdin into a Cursor<Vec<u8>>.
    // I'm not planning on writing big programs, so this works for now.
    // Later on, I can replace the cursor with a file and it should keep working as is.
    let mut program = Vec::new();
    io::stdin().read_to_end(&mut program).expect("could not read from stdin");
    
    let mut compiler = Compiler::new(program);
    compiler.emit();
}
