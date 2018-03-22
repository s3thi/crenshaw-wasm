use std::io;
use std::io::{Read, Cursor};
use std::process;
use std::collections::HashMap;

struct Compiler {
    // The next character we're going to consider.
    lookahead: Option<char>,

    // Our input stream of bytes. Cursor is so dope.
    input_stream: Cursor<Vec<u8>>,

    // Our variables.
    bindings: HashMap<String, i32>,
}

impl Compiler {
    fn new(program: Vec<u8>) -> Compiler {
        Compiler {
            lookahead: None,
            input_stream: Cursor::new(program),
            bindings: HashMap::new(),
        }
    }

    /// Initializes the compiler by reading a single character into the
    /// lookahead, and skipping any whitespace.
    fn init(&mut self) {
        self.get_char();
        self.skip_whitespace();
    }

    /// Runs the compiler.
    fn emit(&mut self) {
        self.init();

        loop {
            self.parse_assignment();
            self.consume_newline();

            if self.lookahead.expect("must end program with a single . on a new line") == '.' {
                break;
            }
        }
    }

    fn consume_newline(&mut self) {
        if self.lookahead.expect("expected a newline") == '\n' {
            self.get_char();
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
            None => {
                self.lookahead = None;
                return None;
            }
        };

        // Convert the byte into an ASCII character.
        self.lookahead = Some(char::from(byte));
        self.lookahead
    }

    /// Consumes spaces until a non-space character is found.
    fn skip_whitespace(&mut self) {
        loop {
            let lookahead = self.lookahead.expect("expected whitespace");
            if lookahead == ' ' {
                self.get_char();
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
    fn expected(&self, what: &str) {
        self.abort(&format!("expected {}", what));
    }

    /// If the current lookahead is not equal to the matching character,
    /// prints an error and exits. Otherwise, consumes another character from
    /// the input stream, puts it in the lookahead, and returns it.
    fn consume_exact_char(&mut self, c: char) -> Option<char> {
        let lookahead = self.lookahead.expect(&format!("expected {}", c));
        if lookahead == c {
            self.get_char();
            self.skip_whitespace();
            Some(lookahead)
        } else {
            self.expected(&c.to_string());
            None
        }
    }

    fn consume_name(&mut self) -> String {
        let mut name = String::from("");
        
        loop {
            let lookahead = self.lookahead.expect("expected ascii character");
            if lookahead.is_ascii_alphanumeric() {
                name.push(lookahead);
                self.get_char();
            } else {
                break;
            }
        }

        self.skip_whitespace();
        name
    }

    /// If the current lookahead is not a digit, prints an error and exits.
    /// Otherwise, consumes another byte from the input stream, puts it in the
    /// lookahead, and returns it.
    fn consume_num(&mut self) -> Option<i32> {
        let mut num = String::from("");
        
        loop {
            let lookahead = self.lookahead.expect("expected number");
            if lookahead.is_digit(10) {
                num.push(lookahead);
                self.get_char();
            } else {
                break;
            }
        }

        self.skip_whitespace();

        match num.parse::<i32>() {
            Ok(n) => Some(n - 0),
            Err(_) => None
        }
    }

    fn parse_assignment(&mut self) -> String {
        let name = self.consume_name();
        self.consume_exact_char('=');
        let expression_value = self.parse_expression();
        self.bindings.insert(name.clone(), expression_value);
        name
    }

    fn parse_factor(&mut self) -> i32 {
        let lookahead = self.lookahead.expect("expected factor");
        let value;

        if lookahead == '(' {
            self.consume_exact_char('(');
            value = self.parse_expression();
            self.consume_exact_char(')');
        } else if lookahead.is_ascii_alphabetic() {
            let name = self.consume_name();
            value = *(self.bindings.get(&name).expect("could not find binding"));
        } else {
            value = self.consume_num().expect("expecter number while parsing factor");
        }

        value
    }

    fn parse_term(&mut self) -> i32 {
        let mut value = self.parse_factor();

        loop {
            let lookahead = self.lookahead.expect("found nothing while parsing term");

            if lookahead == '*' || lookahead == '/' {
                match lookahead {
                    '*' => {
                        self.consume_exact_char('*');
                        value = value * self.parse_factor();
                    },
                    '/' => {
                        self.consume_exact_char('/');
                        value = value / self.parse_factor();
                    },
                    _ => break
                }
            } else {
                break;
            }
        }

        value
    }
    
    fn parse_expression(&mut self) -> i32 {
        let mut value;
        
        let lookahead = self.lookahead.expect("found nothing while parsing start of expression");
        if lookahead == '+' || lookahead == '-' {
            value = 0;
        } else {
            value = self.parse_term();
        }

        loop {
            self.skip_whitespace();
            let lookahead = self.lookahead.expect("found nothing while parsing expression");
            if lookahead == '+' || lookahead == '-' {
                match lookahead {
                    '+' => {
                        self.consume_exact_char('+');
                        value = value + self.parse_term();
                    },
                    '-' => {
                       self.consume_exact_char('-');
                       value = value - self.parse_term();
                    },
                    _ => break
                }
            } else {
                break;
            }
        }

        value
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
