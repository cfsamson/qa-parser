type AppErr = &'static str;

#[derive(Debug)]
pub struct Parser {
    input: Vec<char>,
    cursor: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect::<Vec<char>>(),
            cursor: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Span>, String> {
        let mut spans = vec![];

        loop {
            match self.block(false) {
                Ok(span_res) => {
                    match span_res {
                        Some(span) => spans.push(span),
                        None => break,
                    }
                }

                Err(e) => {
                    let formatted_e = self.report_err(e);
                    return Err(formatted_e);
                },
            }
        }

        Ok(spans)
    }

    fn block(&mut self, sub: bool) -> Result<Option<Span>, AppErr> {
        // This is just for debugging convenience, paste this to see the state of the parser
        // println!("cursor: {}\n{}", self.cursor, &self.input[self.cursor..].iter().collect::<String>());
        
        // Sales (
        let block_start = match self.block_start() {
            Ok(name) => name,
            Err(e) => return Err(e),
        };

        let name = match block_start {
            Some(name) => name,
            None => return Ok(None),
        };

        // *' ' | '\n' * n..y *i \n
        let mut ranges: Vec<Range> = vec![];
        loop {
            match self.range()? {
                Some(range) => ranges.push(range),
                None => break,
            }
        }

        

        // * ' ' (
        let mut subspans = vec![];
        while let Some(span) = self.block(true)? {
            subspans.push(span);
        }

        

        // ) => *char
        let mut sum_name = self.block_end()?;

        let sumtype = if sub {
            SumType::SubTotal(sum_name)
        } else {
            SumType::SumTotal(sum_name)
        };

        let span = Span {
            name,
            ranges,
            subspans,
            sum_type: sumtype,
        };

        
        Ok(Some(span))
    }

    /// ) => *char \n
    fn block_end(&mut self) -> Result<Option<String>, AppErr> {
        let mut name = String::new();
        let mut is_block_end = false;

        self.skip_ws_and_nl();
        while let Some(c) = self.next() {
            match c {
                ')' => {
                    while let Some(ch) = self.next() {
                        match ch {
                            ' ' => (),
                            '=' => match self.peek(1) {
                                Some('>') => {
                                    let _ = self.next();
                                    is_block_end = true;
                                    break;
                                }

                                Some(next_ch) => return Err("Expected >"),
                                _ => return Err("Expected => after )"),
                            },
                            _ => return Ok(None),
                        }
                    }
                }

                _ => break,
            }
        }

        if !is_block_end {
            return Ok(None)
        }

        // We know that we have ) =>

        self.skip_ws();
        let mut skip_ws = true;

        while let Some(c) = self.next() {
            match c {
                '\n' => {
                    if !skip_ws {
                        break;
                    }
                },
                '\r' => match self.peek(1) {
                    Some('\n') => {
                        let _ = self.next();
                        if !skip_ws {
                            break;
                        }
                    }
                    _ => {
                        skip_ws = false;
                        name.push(c);
                    },
                },

                _ => {
                    skip_ws = false;
                    name.push(c);
                },
            }
        }

        // remove any trailing whitespace
        let name = name.trim_end().to_string();

        Ok(Some(name))
    }

    /// chars*(
    /// Returns an error if there is a parse error in a block.
    /// The next is an Option which indicates if there is a "block start" or not
    /// The last option is to indicate if there is a title/header for the block or not
    fn block_start(&mut self) -> Result<Option<Option<String>>, AppErr> {
        let mut skip_ws = true;
        let mut name = String::new();
        let mut is_block_start = false;
        let mut lookahed = 1;
        while let Some(c) = self.peek(lookahed) {
            match c {
                '(' => {
                    is_block_start = true;
                    break;
                },

                ')' | '=' => {
                    // we need to move the cursor for correct error reporting
                    return Ok(None)
                },

                _ => (),
            }

            lookahed += 1;
        }

        // if we got all the way to the end without finding a `(` we know this is not a block
        // but it's not an error
        if !is_block_start {
            return Ok(None);
        }

        self.skip_ws_and_nl();
        while let Some(c) = self.next() {
            //println!("{:?}", self);
            // println!("{:?}", c);
            match c {
                '(' => break,
                _ => name.push(c),
            }
        }

       
        if name.is_empty() {
            Ok(Some(None))
        } else {
            let name = name.trim_end().to_string();
            Ok(Some(Some(name)))
        }
    }

    fn is_space_or_newline(c: char) -> bool {
        c.is_whitespace() || c.is_control()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek(1) {
            if c.is_whitespace() {
                let _ = self.next();
            } else {
                break;
            }
        }
    }

    fn skip_ws_and_nl(&mut self) {
        while let Some(c) = self.peek(1) {
            if Parser::is_space_or_newline(c) {
                let _ = self.next();
            } else {
                break;
            }
        }
    }
    /// int* .. int* ' '* => ' '* char* /n
    fn range(&mut self) -> Result<Option<Range>, AppErr> {
        // 1111
        self.skip_ws_and_nl();
        let range_start = match self.check_range_part()? {
            Some(range) => range,
            None => return Ok(None),
        };

        // ..
        for _ in 0..2 {
            match self.next().unwrap() {
                '.' => (),
                _ => {
                    // we need to decrease the cursor since we already moved past the error
                    self.cursor -= 1;
                    return Err("Invalid range syntax");
                },
            }
        }

        // 1111
        let range_end = match self.check_range_part()? {
            Some(range) => range,
            None => return Err("Invalid range"),
        };

        // =>
        self.skip_ws();
        while let Some(c) = self.next() {
            match c {
                '=' => match self.peek(1) {
                    Some('>') => {
                        let _ = self.next();
                        break;
                    }
                    Some(c) => {
                        return Err("Invalid syntax after =");
                    },
                    None => return Err("Unexpected EOF"),
                },

                _ => return Err("Unexpected syntax"),
            }
        }

        // Title
        let mut title = String::new();
        self.skip_ws();
        while let Some(c) = self.next() {
            match c {
                '\n' => break,

                '\r' => {
                    if let Some(c) = self.peek(1) {
                        if c == '\n' {
                            self.next();
                            break;
                        } else {
                            title.push(self.next().unwrap());
                        }
                    }
                }

                _ => title.push(c),
            }
        }

        // remove any trailing spaces
        let title = title.trim_end().to_string();

        let from: u32 = range_start.parse().expect("Not a number");
        let to: u32 = range_end.parse().expect("Not a number");

        let range = Range { title, from, to };

        Ok(Some(range))
    }

    fn check_range_part(&mut self) -> Result<Option<String>, AppErr> {
        let mut from = String::new();

        let rangeint = match self.peek(1) {
            Some(r) => r,
            None => return Ok(None),
        };


        if rangeint.is_numeric() {
            from.push(self.next().unwrap());
        } else {
            return Ok(None);
        }

        while self.peek(1).unwrap().is_numeric() {
            from.push(self.next().unwrap());
        }
        Ok(Some(from))
    }

    fn next(&mut self) -> Option<char> {
        let c = self.input.get(self.cursor).map(|c| *c);
        self.cursor += 1;
        c
    }

    fn peek(&self, n: usize) -> Option<char> {
        self.input.get(self.cursor + n - 1).map(|c| *c)
    }

    fn report_err(&self, msg: &str) -> String {
        let (line, charpos, line_start_pos) = self
        .input.iter()
        .take(self.cursor)
        .fold((0, 0, 0), |acc, ch| {
            if *ch == '\n' {
                let nl_pos = acc.2 + acc.1 + 1;
                (acc.0 + 1, 0, nl_pos)
            } else {
                (acc.0, acc.1 + 1, acc.2)
            }
        });

        //println!("line: {}, charpos: {}, lsp: {}", line, charpos, line_start_pos);

        let mut text = String::new();
        let mut indicator = String::new();

        for (i, ch) in self.input[line_start_pos..].iter().enumerate() {
            match *ch {
                '\n' => break,
                _ => {
                    text.push(*ch);
                    let pos = line_start_pos + i;
                    if pos < self.cursor {
                        indicator.push('-');
                    } else if pos == self.cursor {
                        indicator.push('^');
                    }
                }
            }
        }

        format!("\nline: {}, pos: {}\n{}\n{}\n\nERROR: {}\n", line + 1, charpos, text, indicator, msg)
    }
}


#[derive(Debug)]
pub struct Range {
    pub title: String,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug)]
pub struct Span {
    pub name: Option<String>,
    pub ranges: Vec<Range>,
    pub subspans: Vec<Span>,
    pub sum_type: SumType,
}

#[derive(Debug)]
pub enum SumType {
    SumTotal(Option<String>),
    SubTotal(Option<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: &str = "
    Sales (
        3010..3010 => Webshop
        3010..4000 => Other sales
    ) => Sum sales
    
    (
        4000..5000 => Material
    ) => Sum material
    
    (
        5000..5000 => Direct labor
        5010..6000 => Other labor costs
    ) => Sum labor costs
    
    Other costs (
        6000..6010 => Leasing
        (
            6020..6100 => Office supplies
            6100..6200 => Consumables
        ) => Sum miscellaneous costs
    ) => Sum other costs
    ";

    #[test]
    fn parse_full_syntax() {
        let mut parser = Parser::new(TEST);

        match parser.parse() {
            Ok(ast) => println!("{:#?}", ast),
            Err(e) => println!("{}", e),
         }
    }

    #[test]
    fn parse_nameless_span_with_sub() {
        let test = "
        Other costs (
            6000..6010 => Leasing
            (
                6020..6100 => Office supplies
                6100..6200 => Consumables
            ) => Sum miscellaneous costs
        ) => Sum other costs
        ";

        let mut parser = Parser::new(test);
        match parser.parse() {
            Ok(ast) => println!("{:?}", ast),
            Err(e) => println!("{}", e),
         }
    }

    #[test]
    fn reports_errors() {
        let test = "
        Other costs (
            6000..6010 => Leasing
            (
                6020..6100 => Office supplies
                6100..6200 => Consumables
            ) => Sum miscellaneous costs
        ) == Sum other costs
        ";

        let mut parser = Parser::new(test);
        match parser.parse() {
            Ok(_) => (),
            Err(e) => println!("{}", e),
         }
        
    }


    #[test]
    fn reports_start_err() {
        let test = "
        )
            6000..6010 => Husleie
            (
                6020..6100 => Småanskaffelser
                6100..6200 => Forbruksartikler
            ) => Sum anskaffelser
        ) == Sum andre kostnader
        ";

        let mut parser = Parser::new(test);
        match parser.parse() {
            Ok(_) => (),
            Err(e) => println!("{}", e),
         }
    }

    #[test]
    fn reports_range_err() {
        let test = "
        (
            6000..6010 => Husleie
            (
                6020.6100 => Småanskaffelser
                6100..6200 => Forbruksartikler
            ) => Sum anskaffelser
        ) == Sum andre kostnader
        ";

        let mut parser = Parser::new(test);
        match parser.parse() {
            Ok(_) => (),
            Err(e) => println!("{}", e),
         }
    }
}
