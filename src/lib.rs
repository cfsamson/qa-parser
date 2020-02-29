type AppErr = &'static str;

#[derive(Debug)]
struct Parser {
    input: Vec<char>,
    cursor: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect::<Vec<char>>(),
            cursor: 0,
        }
    }

    fn parse(&mut self) -> Result<Vec<Span>, AppErr> {
        let mut spans = vec![];

        while let Some(span) = self.block()? {
            spans.push(span);
        }

        Ok(spans)
    }

    fn block(&mut self) -> Result<Option<Span>, AppErr> {
        // Salg (
        let name = match self.block_start() {
            Ok(name) => name,
            Err(e) => return Ok(None),
        };

        // *' ' | '\n' * n..y *i \n
        let mut ranges: Vec<Range> = vec![];
        loop {
            match self.range()? {
                Some(range) => ranges.push(range),
                None => break,
            }
        }

        // ) => *char


        // * ' ' (
        let mut subspans = vec![];
        while let Some(span) = self.block()? {
            subspans.push(span);
        }

        let mut sum_name = self.block_end()?;

        let span = Span {
            name,
            ranges,
            subspans,
            sum_type: SumType::SubTotal(sum_name),
        };

        
        Ok(Some(span))
    }

    /// ) => *char \n
    fn block_end(&mut self) -> Result<Option<String>, AppErr> {
        let mut name = String::new();
        let mut is_block_end = false;

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
        let mut skip_ws = true;

        while let Some(c) = self.next() {
            match c {
                ' ' => {
                    if !skip_ws {
                        name.push(c);
                    }
                }

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


        Ok(Some(name))
    }

    // chars*(
    fn block_start(&mut self) -> Result<Option<String>, AppErr> {
        let mut skip_ws = true;
        let mut name = String::new();
        let mut is_block_start = false;

        let mut lookahed = 1;
        while let Some(c) = self.peek(lookahed) {
            dbg!(c);
            match c {
                '(' => {
                    is_block_start = true;
                    break;
                },

                ')' | '=' => break,

                _ => (),
            }

            lookahed += 1;
        }

        // if it's not a valid block start
        if !is_block_start {
            return Err("Not a block");
        }

        while let Some(c) = self.next() {
            //println!("{:?}", self);
            // println!("{:?}", c);
            match c {
                ' ' => {
                    if !skip_ws {
                        name.push(c);
                    }
                }

                '\n' => {
                    if !skip_ws {
                        break;
                    }
                }

                '(' => break,

                _ => {
                    skip_ws = false;
                    name.push(c);
                }

            }
        }

        println!("{:?}", name);
        
       
        if name.is_empty() {
            Ok(None)
        } else {
            Ok(Some(name))
        }
    }
    /// int* .. int* ' '* => ' '* char* /n
    fn range(&mut self) -> Result<Option<Range>, AppErr> {
        // 1111

        while let Some(c) = self.peek(1) {
            match c {
                ' ' | '\n' => {
                    let _ = self.next();
                },
                _ => break,
            }
        }

        let range_start = match self.check_range_part()? {
            Some(range) => range,
            None => return Ok(None),
        };

        // ..
        for _ in 0..2 {
            match self.next().unwrap() {
                '.' => (),
                _ => return Err("Invalid range syntax"),
            }
        }

        // 1111
        let range_end = match self.check_range_part()? {
            Some(range) => range,
            None => return Err("Invalid range"),
        };

        // =>
        while let Some(c) = self.next() {
            match c {
                ' ' => (),
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
        let mut skip_ws = true;
        while let Some(c) = self.next() {
            match c {
                ' ' => {
                    if skip_ws {
                        ()
                    } else {
                        title.push(c);
                    }
                }

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

        let from: u32 = range_start.parse().expect("Not a number");
        let to: u32 = range_end.parse().expect("Not a number");

        let range = Range { title, from, to };

        Ok(Some(range))
    }

    // //' '* ) ' '* => char* /n
    // fn block_end(&mut self) -> Result<(), AppErr> {
    //     while let Some(c) = self.next() {
    //         ' ' => (),
    //         ')' => return Ok(()),
    //         _ => return Err("Invalid end of span"),
    //     }

    //     Err("Expected ), got EOF.")
    // }

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
}

#[derive(Debug)]
struct Range {
    title: String,
    from: u32,
    to: u32,
}

#[derive(Debug)]
struct Span {
    name: Option<String>,
    ranges: Vec<Range>,
    subspans: Vec<Span>,
    sum_type: SumType,
}

#[derive(Debug)]
enum SumType {
    SumTotal(Option<String>),
    SubTotal(Option<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: &str = "
    Salg (
        3010..3010 => Nettbutikk
        3010..4000 => Annet salg
    ) => Sum salg
    
    (
        4000..5000 => Varer
    ) => Sum Varer
    
    (
        5000..5000 => Ordinær lønn
        5010..6000 => Annen lønn
    ) => Sum lønnskostnader
    
    (
        6000..6010 => Husleie
        (
            6020..6100 => Småanskaffelser
            6100..6200 => Forbruksartikler
        ) => Sum anskaffelser
        6200..7000 => Diverse
    ) => Sum andre kostnader
    ";

    #[test]
    fn it_works() {
        let mut parser = Parser::new(TEST);

        let ast = parser.parse();

        println!("{:#?}", ast);
    }
}
