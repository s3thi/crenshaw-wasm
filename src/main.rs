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

        // Ignore newlines for now.
        if char::from(byte) == '\n' {
            self.get_char()
        } else {
            self.lookahead
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

        Some(name)
    }

    /// If the current lookahead is not a digit, prints an error and exits.
    /// Otherwise, consumes another byte from the input stream, puts it in the
    /// lookahead, and returns it.
    fn consume_num(&mut self) -> Option<char> {
        if let Some(lookahead) = self.lookahead {
            if lookahead.is_digit(10) {
                self.get_char();
                Some(lookahead)
            } else {
                self.expected("integer", &lookahead.to_string());
                None
            }
        } else {
            self.expected("integer", "nothing");
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
        println!("(return)");
        println!(")");
        println!("(export \"main\" (func $main))");
    }

    fn parse_assignment(&mut self) {
        let name = self.consume_name();

        if let Some(name) = name {
            println!("(local ${} i32)", name);
            self.consume_exact_char('=');
            println!("(set_local ${}", name);
            self.parse_expression();
            println!(")");
            println!("(get_local ${})", name);
        } else {
            self.expected("identifier", "nothing");
        }
    }
    
    /// <expression> ::= <leading> <term> <addop>
    /// <leading> ::= "+" | "-" | ""
    /// <addop> ::= <add-expression> | <subtract-expression>
    fn parse_expression(&mut self) {
        
        if let Some(c) = self.lookahead {
            if c == '+' {
                self.consume_exact_char('+');
                self.parse_term();
            } else if c == '-' {
                println!("(i32.const 0)");
            } else {
                self.parse_term();
            }
        } else {
            self.expected("term or addop", "nothing");
        }

        loop {
            if let Some(c) = self.lookahead {
                if c == '+' {
                    self.parse_add();
                } else if c == '-' {
                    self.parse_subtract();
                } else {
                    break;
                }
            } else {
                self.expected("addop", "nothing");
            }
        }
    }
    
    /// <term> ::= <factor> <multop>
    /// <multop> ::= <multiply-expression> | <divide-expression>
    fn parse_term(&mut self) {
        self.parse_factor();
        loop {
            if let Some(c) = self.lookahead {
                if c == '*' {
                    self.parse_multiply();
                } else if c == '/' {
                    self.parse_divide();
                } else {
                    break;
                }
            } else {
                self.expected("multop", "nothing");
            }
        }
    }

    /// <factor> ::= "(" <expression> ")" | <number>
    fn parse_factor(&mut self) {
        if let Some(c) = self.lookahead {
            if c == '(' {
                self.consume_exact_char('(');
                self.parse_expression();
                self.consume_exact_char(')');
            } else if c.is_ascii_alphabetic() {
                self.parse_identifier();
            } else {
                let num = self.consume_num().unwrap();
                println!("(i32.const {})", num);
            }
        } else {
            self.expected("expression", "nothing");
        }
    }

    fn parse_identifier(&mut self) {
        let name = self.consume_name().unwrap();
        if let Some(c) = self.lookahead {
            if c == '(' {
                self.consume_exact_char('(');
                self.consume_exact_char(')');
                println!("(call ${})", name);
            } else {
                println!("(get_local ${})", name);
            }
        }
    }

    /// <add-expression> ::= <empty> | <plus-term>
    /// <plus-term> ::= "+" <term>
    fn parse_add(&mut self) {
        // Consume a '+' character from the stream.
        self.consume_exact_char('+');
        
        // Call term() again to consume one more term.
        self.parse_term();
        
        // Add the two terms on the stack.
        println!("(i32.add)");
    }

    /// <subtract-expression> ::= <empty> | <minus-term>
    /// <minus-term> ::= "-" <term>
    fn parse_subtract(&mut self) {
        self.consume_exact_char('-');
        self.parse_term();
        println!("(i32.sub)");
    }

    /// <multiply-expression> ::= <empty> | <multiply-factor>
    /// <multiply-factor> ::= "*" <factor>
    fn parse_multiply(&mut self) {
        self.consume_exact_char('*');
        self.parse_factor();
        println!("(i32.mul)");
    }

    /// <divide-expression> ::= <empty> | <divide-factor>
    /// <divide-factor> ::= "/" <factor>
    fn parse_divide(&mut self) {
        self.consume_exact_char('/');
        self.parse_factor();
        println!("(i32.div_s)");
    }
}

fn main() {
    // I slurp up everything from stdin into a Cursor<Vec<u8>>.
    // I'm not planning on writing big programs, so this works for now.
    // Later on, I can replace the cursor with a file and it should keep working as is.
    let mut program = Vec::new();
    io::stdin().read_to_end(&mut program).expect("could not read from stdin");
    
    let mut compiler = Compiler::new(program);
    compiler.get_char();
    compiler.emit_module_start();
    compiler.emit_main_start();
    compiler.parse_assignment();
    
    if let Some(c) = compiler.lookahead {
        if c != '\n' {
            compiler.expected("newline", &c.to_string());
        }
    } else {
        compiler.expected("newline", "nothing");
    }
    
    compiler.emit_main_end();
    compiler.emit_module_end();
}
