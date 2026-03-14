pub(crate) struct Tokenizer<'a> {
    #[allow(dead_code)]
    input: &'a str,
    iterator: std::str::Chars<'a>,
    pub(crate) line_ended: bool,
}

impl<'a> Tokenizer<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(config: &'a str) -> Self {
        Self {
            input: config,
            iterator: config.chars(),
            line_ended: true,
        }
    }

    #[allow(unused)]
    pub(crate) fn reset(&mut self) {
        self.iterator = self.input.chars();
        self.line_ended = true;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (String, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let mut token = String::new();
        self.line_ended = false;
        let mut quote = false;
        let mut escape = false;
        let mut pure = true;
        while let Some(chr) = self.iterator.next() {
            if escape {
                token.push(chr);
                escape = false;
                continue;
            }
            match chr {
                '#' => {
                    while self.iterator.next().unwrap_or('\n') != '\n' {}
                    if !token.is_empty() {
                        self.line_ended = true;
                        break;
                    }
                }
                '"' => {
                    pure = false;
                    quote = !quote;
                }
                ' ' => {
                    if quote {
                        token.push(chr)
                    } else if !token.is_empty() {
                        break;
                    }
                }
                '\\' => {
                    pure = false;
                    escape = true;
                }
                '\n' => {
                    if !token.is_empty() {
                        self.line_ended = true;
                        break;
                    }
                }
                _ => token.push(chr),
            }
        }

        if token.is_empty() {
            None
        } else {
            Some((token, pure))
        }
    }
}
