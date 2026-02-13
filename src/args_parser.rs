pub struct ArgsParser {
    chars: Vec<char>,
    index: usize,
    reading_string: bool,
    quotes_type: char,
    outside_string_space: bool,
}

impl ArgsParser {
    pub fn new(args: &str) -> Self {
        Self {
            chars: args.chars().collect::<Vec<char>>(),
            index: 0,
            reading_string: false,
            quotes_type: '\'',
            outside_string_space: false,
        }
    }

    pub fn parse(&mut self) -> Vec<String> {
        let mut arguments: Vec<String> = Vec::new();
        let mut buffer = String::new();
        let mut i = 0;

        while i < self.chars.len() {
            let c = self.chars[i];

            match c {
                '\\' if i + 1 < self.chars.len() => {
                    buffer.push(self.chars[i + 1]);
                    i += 2;
                    continue;
                }
                '\'' | '"' => {
                    if self.reading_string {
                        if self.quotes_type == c {
                            self.reading_string = false;
                            let arg = if buffer.is_empty() {
                                " ".to_string()
                            } else {
                                buffer.clone()
                            };

                            if arg != " " {
                                arguments.push(arg);
                            }
                            buffer.clear();
                        } else {
                            buffer.push(c);
                        }
                    } else {
                        if !buffer.is_empty() {
                            arguments.push(buffer.clone());
                            buffer.clear();
                        }
                        self.reading_string = true;
                        self.quotes_type = c;
                    }
                }
                ' ' if !self.reading_string => {
                    if !buffer.is_empty() && !buffer.ends_with(' ') {
                        arguments.push(buffer.clone());
                        buffer.clear();
                    }
                    if !buffer.ends_with(' ') {
                        buffer.push(c);
                    }
                }
                '\n' if !self.reading_string => {
                    if !buffer.is_empty() {
                        arguments.push(buffer.clone());
                        buffer.clear();
                    }
                }
                _ => {
                    buffer.push(c);
                    self.outside_string_space = c == ' ';
                }
            }
            i += 1;
        }
        
        if !buffer.is_empty() {
            arguments.push(buffer);
        }
        arguments
    }
}
