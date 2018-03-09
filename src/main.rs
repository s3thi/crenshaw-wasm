use std::io;
use std::io::{Read, Cursor};
use std::process;

struct Compiler {
    // The next character we're going to consider.
    lookahead: Option<char>,

    // Our input stream of bytes.
    input_stream: Cursor<Vec<u8>>,
}

impl Compiler {
    fn new(program: Vec<u8>) -> Compiler {
        Compiler {
            lookahead: None,
            input_stream: Cursor::new(program),
        }
    }

    /// Consumes the next byte in the stream, converts it to a character,
    /// stores it in the lookahead, and returns the character.
    fn get_char(&mut self) -> Option<char> {
        // Read a single byte from the stream.
        let mut buf = [0];
        let result = self.input_stream.read_exact(&mut buf).ok();
        let byte = match result {
            Some(_) => buf[0],
            None => return None
        };

        // Convert the byte into an ASCII character.
        self.lookahead = Some(char::from(byte));
        self.lookahead
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
    fn expected(&self, what: &str) {
        self.abort(&format!("expected {}", what));
    }

    /// If the current lookahead is not equal to the matching character,
    /// prints an error and exits. Otherwise, consumes another byte from
    /// the input stream.
    fn match_char(&mut self, c: char) {
        if let Some(lookahead) = self.lookahead {
            if lookahead == c {
                self.get_char();
            } else {
                self.expected(&format!("{}", c));
            }
        } else {
            self.expected(&format!("{}", c));
        }
    }

    fn get_name(&mut self) -> Option<char> {
        if let Some(lookahead) = self.lookahead {
            if lookahead.is_ascii_alphanumeric() {
                self.get_char();
                Some(lookahead.to_ascii_uppercase())
            } else {
                self.expected("name");
                None
            }
        } else {
            self.expected("name");
            None
        }
    }

    /// If the current lookahead is not a digit, prints an error and exits.
    /// Otherwise, consumes another byte from the input stream and returns the
    /// matched digit.
    fn get_num(&mut self) -> Option<char> {
        if let Some(lookahead) = self.lookahead {
            if lookahead.is_digit(10) {
                self.get_char();
                Some(lookahead)
            } else {
                self.expected("integer");
                None
            }
        } else {
            self.expected("integer");
            None
        }
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
        println!(")");
        println!("(export \"main\" (func $main))");
    }

    fn term(&mut self) {
        // A term is simply a number.
        let num = self.get_num().unwrap();
        println!("(i32.const {})", num);
    }

    fn expression(&mut self) {
        // An expression is a term, followed by an addop, followed by
        // another term.
        self.term();

        if let Some(c) = self.lookahead {
            if c == '+' {
                self.add();
            } else if c == '-' {
                self.subtract();
            } else {
                self.expected("addop");
            }
        } else {
            self.expected("addop");
        }
    }

    fn add(&mut self) {
        // Consume a '+' character from the stream.
        self.match_char('+');
        
        // Call term() again to consume one more term.
        self.term();
        
        // Add the two terms on the stack.
        println!("(i32.add)");
    }

    fn subtract(&mut self) {
        self.match_char('-');
        self.term();
        println!("(i32.sub)");
    }
}

fn main() {
    // I slurp up everything from stdin into a Cursor<Vec<u8>>.
    // I'm not planning on writing big programs, so this works for now.
    // Later on, I can replace the cursor with a file and it should keep working as is.
    let mut program = Vec::new();
    io::stdin().read_to_end(&mut program).expect("could not read from stdin");
    
    let mut c = Compiler::new(program);
    c.get_char();
    c.emit_module_start();
    c.emit_main_start();
    c.expression();
    c.emit_main_end();
    c.emit_module_end();
}
