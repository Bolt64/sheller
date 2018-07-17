extern crate nix;
extern crate void;

use std::ffi::CString;
use std::iter::FromIterator;
use lexer::void::Void;
use lexer::nix::unistd::*;

#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    Progname(&'a str),
    Argument(&'a str),
    Separator,
    Quit,
}

#[derive(PartialEq, Debug)]
pub enum ParseError<'a> {
    UnbalancedQuote(&'a str),
    QuoteInAtomicString(&'a str),
    TokenOutOfPlace,
    NullByteError,
}

#[derive(PartialEq, Debug)]
pub struct Command {
    progname: CString,
    arguments: Vec<CString>,
}

impl Command {
    pub fn new(tokens: Vec<Token>) -> Result<Command, ParseError> {
        let mut cstrings: Vec<CString> = Vec::new();
        for (index, token) in tokens.iter().enumerate() {
            if index == 0 {
                match token {
                    Token::Progname(string) => match CString::new(*string) {
                        Ok(cstring) => cstrings.push(cstring),
                        Err(_) => return Err(ParseError::NullByteError),
                    },
                    _ => return Err(ParseError::TokenOutOfPlace),
                }
            } else {
                match token {
                    Token::Argument(string) => match CString::new(*string) {
                        Ok(cstring) => cstrings.push(cstring),
                        Err(_) => return Err(ParseError::NullByteError),
                    },
                    _ => return Err(ParseError::TokenOutOfPlace),
                }
            }
        }
        let progname = cstrings[0].clone();
        let arguments = Vec::from_iter(cstrings[1..].iter().cloned());
        Ok(Command {
            progname,
            arguments,
        })
    }

    pub fn execute(&self) -> nix::Result<Void> {
        execvp(&self.progname, &self.arguments)
    }
}

#[derive(Debug)]
enum Function {
    ShellCommand(Command),
    Quit,
}

/* This function takes a string slice, and tries to lex it according
to a very basic grammar.
It first splits up the string into the atomic commands, i.e. the commands
separated by ';'. It then calls `tokenize_atomic_string on each of the
atomic strings separately.` */
pub fn tokenize_string(string: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let atomic_strings =
        exclusive_separate_at_positions(string, find_all_separator_positions(string));
    for (index, atomic_string) in atomic_strings.iter().enumerate() {
        if index > 0 {
            tokens.push(Token::Separator);
        }
        let mut atomic_tokens = tokenize_atomic_string(atomic_string)?;
        tokens.append(&mut atomic_tokens);
    }
    Ok(tokens)
}

/* This function does most of the heavy lifting. It first checks for
any quoted portion in the atomic string. If there is any, it checks
it for balanced quotes, and finally, splits up the line into tokens. */
fn tokenize_atomic_string(string: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let quoted_block_position = find_quoted_block_position(string)?;
    match quoted_block_position {
        Some((fb, lb)) => {
            let left_words = split_by_whitespace(&string[..fb])?;
            let right_words = split_by_whitespace(&string[(lb + 1)..])?;

            for (index, word) in left_words.iter().enumerate() {
                if index == 0 {
                    match word.to_owned() {
                        "quit" => tokens.push(Token::Quit),
                        _ => tokens.push(Token::Progname(word)),
                    };
                } else {
                    tokens.push(Token::Argument(word));
                }
            }

            let quoted_part = Token::Argument(&string[fb..(lb + 1)]);
            tokens.push(quoted_part);

            for word in right_words.iter() {
                tokens.push(Token::Argument(word));
            }
        }
        None => {
            let words = split_by_whitespace(string)?;
            for (index, word) in words.iter().enumerate() {
                if index == 0 {
                    match word.to_owned() {
                        "quit" => tokens.push(Token::Quit),
                        _ => tokens.push(Token::Progname(word)),
                    };
                } else {
                    tokens.push(Token::Argument(word));
                }
            }
        }
    }
    Ok(tokens)
}

/* This function splits up a string and ignores the whitespace.
If it comes across a quote, it throws a ParseError. */
fn split_by_whitespace(string: &str) -> Result<Vec<&str>, ParseError> {
    let mut individual_words: Vec<&str> = Vec::new();
    let quote_position = string.find('\"');
    match quote_position {
        Some(pos) => Err(ParseError::QuoteInAtomicString(&string[pos..])),
        None => {
            for string_slice in string.split_whitespace() {
                individual_words.push(string_slice);
            }
            Ok(individual_words)
        }
    }
}

/* This function actually finds the quotes position and checks for balanced
quotes, throwing an error otherwise. */
fn find_quoted_block_position(string: &str) -> Result<Option<(usize, usize)>, ParseError> {
    let first_occurrence = string.find('\"');
    let last_occurrence = string.rfind('\"');
    match (first_occurrence, last_occurrence) {
        (None, _) => Ok(None),
        (_, None) => Ok(None),
        (Some(fb), Some(lb)) => {
            if has_balanced_apostrophes(&string[fb..(lb + 1)], '\"') {
                Ok(Some((fb, lb)))
            } else {
                Err(ParseError::UnbalancedQuote(&string[fb..lb + 1]))
            }
        }
    }
}

/* This function does the actual checking for balanced apostrophes */
fn has_balanced_apostrophes(string: &str, apostrophe_char: char) -> bool {
    let first_occurrence = string.find(apostrophe_char);
    let last_occurrence = string.rfind(apostrophe_char);
    match (first_occurrence, last_occurrence) {
        (None, _) => true,
        (_, None) => true,
        (Some(fb), Some(lb)) => {
            if fb == lb {
                false
            } else {
                has_balanced_apostrophes(&string[(fb + 1)..(lb)], apostrophe_char)
            }
        }
    }
}

/* The next two functions find the positions of the separator, and
separate the string according to them. I didn't use the string.split(';')
method because that wouldn't distinguish ';' from an escaped '\;'. */
fn find_all_separator_positions(string: &str) -> Vec<usize> {
    let mut separator_positions = Vec::new();
    for (index, character) in string.chars().enumerate() {
        if character == ';' {
            if index > 0 && string.chars().nth(index - 1).unwrap() == '\\' {
                if index > 1 && string.chars().nth(index - 2).unwrap() == '\\' {
                    separator_positions.push(index);
                }
            } else {
                separator_positions.push(index);
            }
        }
    }
    separator_positions
}

fn exclusive_separate_at_positions(string: &str, positions: Vec<usize>) -> Vec<&str> {
    let mut current_start: usize = 0;
    let mut separated_slices: Vec<&str> = Vec::new();

    for position in positions.iter() {
        separated_slices.push(&string[current_start..*position]);
        current_start = position + 1;
    }
    separated_slices.push(&string[current_start..]);
    separated_slices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_new_test() {
        let string = "ls";
        let tokens = tokenize_string(&string).unwrap();
        let progname = CString::new("ls").unwrap();
        let arguments: Vec<CString> = Vec::new();
        let expected_result = Ok(Command {
            progname,
            arguments,
        });
        assert_eq!(Command::new(tokens), expected_result);

        let string = "ls -l";
        let tokens = tokenize_string(&string).unwrap();
        let progname = CString::new("ls").unwrap();
        let arguments = vec![CString::new("-l").unwrap()];
        let expected_result = Ok(Command {
            progname,
            arguments,
        });
        assert_eq!(Command::new(tokens), expected_result);

        let string = "echo 3 \"more\"";
        let tokens = tokenize_string(&string).unwrap();
        let progname = CString::new("echo").unwrap();
        let arguments = vec![
            CString::new("3").unwrap(),
            CString::new("\"more\"").unwrap(),
        ];
        let expected_result = Ok(Command {
            progname,
            arguments,
        });
        assert_eq!(Command::new(tokens), expected_result);

        let string = "quit";
        let tokens = tokenize_string(&string).unwrap();
        let expected_result = Err(ParseError::TokenOutOfPlace);
        assert_eq!(Command::new(tokens), expected_result);

        let string = "str\0ing";
        let tokens = tokenize_string(&string).unwrap();
        let expected_result = Err(ParseError::NullByteError);
        assert_eq!(Command::new(tokens), expected_result);
    }

    #[test]
    fn tokenize_string_test() {
        let string = ";";
        let expected_result = Ok(vec![Token::Separator]);
        assert_eq!(tokenize_string(&string), expected_result);

        let string = ";;";
        let expected_result = Ok(vec![Token::Separator, Token::Separator]);
        assert_eq!(tokenize_string(&string), expected_result);

        let string = "echo 3 ; quit";
        let expected_result = Ok(vec![
            Token::Progname(&string[0..4]),
            Token::Argument(&string[5..6]),
            Token::Separator,
            Token::Quit,
        ]);
        assert_eq!(tokenize_string(&string), expected_result);
    }

    #[test]
    fn tokenize_atomic_string_test() {
        let string = "echo";
        let expected_result = Ok(vec![Token::Progname(&string[..])]);
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "quit";
        let expected_result = Ok(vec![Token::Quit]);
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "echo quit";
        let expected_result = Ok(vec![
            Token::Progname(&string[0..4]),
            Token::Argument(&string[5..]),
        ]);
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "quit echo";
        let expected_result = Ok(vec![Token::Quit, Token::Argument(&string[5..])]);
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "echo \"blah\"";
        let expected_result = Ok(vec![
            Token::Progname(&string[0..4]),
            Token::Argument(&string[5..]),
        ]);
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "echo \"bl\"h\"";
        let expected_result = Err(ParseError::UnbalancedQuote(&string[5..11]));
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "echo 3 \"blah\" 4";
        let expected_result = Ok(vec![
            Token::Progname(&string[0..4]),
            Token::Argument(&string[5..6]),
            Token::Argument(&string[7..13]),
            Token::Argument(&string[14..15]),
        ]);
        assert_eq!(tokenize_atomic_string(&string), expected_result);

        let string = "  ";
        let expected_result = Ok(Vec::new());
        assert_eq!(tokenize_atomic_string(&string), expected_result);
    }

    #[test]
    fn split_by_whitespace_test() {
        let string = "echo";
        let expected_result = Ok(vec![&string[..]]);
        assert_eq!(split_by_whitespace(&string), expected_result);

        let string = "echo 3";
        let expected_result = Ok(vec![&string[0..4], &string[5..6]]);
        assert_eq!(split_by_whitespace(&string), expected_result);

        let string = "echo   3";
        let expected_result = Ok(vec![&string[0..4], &string[7..8]]);
        assert_eq!(split_by_whitespace(&string), expected_result);

        let string = "echo \"3\"";
        let expected_result = Err(ParseError::QuoteInAtomicString(&string[5..]));
        assert_eq!(split_by_whitespace(&string), expected_result);
    }

    #[test]
    fn find_quoted_block_position_test() {
        let string = "echo 3";
        let expected_result = Ok(None);
        assert_eq!(find_quoted_block_position(&string), expected_result);

        let string = "echo \"";
        let expected_result = Err(ParseError::UnbalancedQuote(&string[5..6]));
        assert_eq!(find_quoted_block_position(&string), expected_result);

        let string = "echo \"floof\"";
        let expected_result = Ok(Some((5, 11)));
        assert_eq!(find_quoted_block_position(&string), expected_result);

        let string = "echo \"floof\"\"";
        let expected_result = Err(ParseError::UnbalancedQuote(&string[5..13]));
        assert_eq!(find_quoted_block_position(&string), expected_result);

        let string = "echo \"fl\"o\"\"";
        let expected_result = Ok(Some((5, 11)));
        assert_eq!(find_quoted_block_position(&string), expected_result);
    }

    #[test]
    fn has_balanced_apostrophes_test() {
        let string = "test string without apostrophe";
        assert_eq!(has_balanced_apostrophes(&string, '\''), true);
        assert_eq!(has_balanced_apostrophes(&string, '\"'), true);

        let string = "test string '";
        assert_eq!(has_balanced_apostrophes(&string, '\''), false);
        assert_eq!(has_balanced_apostrophes(&string, '\"'), true);

        let string = "test 'string \"even more\"'";
        assert_eq!(has_balanced_apostrophes(&string, '\''), true);
        assert_eq!(has_balanced_apostrophes(&string, '\"'), true);
    }

    #[test]
    fn find_all_separator_positions_test() {
        let string = ";";
        let expected_result: Vec<usize> = vec![0];
        assert_eq!(find_all_separator_positions(&string), expected_result);

        let string = "ls ; cat file";
        let expected_result: Vec<usize> = vec![3];
        assert_eq!(find_all_separator_positions(&string), expected_result);

        let string = "ls ; cat file ; echo 3";
        let expected_result: Vec<usize> = vec![3, 14];
        assert_eq!(find_all_separator_positions(&string), expected_result);

        let string = "ls ; cat file \\; echo 3";
        let expected_result: Vec<usize> = vec![3];
        assert_eq!(find_all_separator_positions(&string), expected_result);

        let string = "ls \\\\; cat file \\; echo 3";
        let expected_result: Vec<usize> = vec![5];
        assert_eq!(find_all_separator_positions(&string), expected_result);
    }

    #[test]
    fn exclusive_separate_at_positions_test() {
        let string = "aaaaa";
        let positions: Vec<usize> = Vec::new();
        let expected_result = vec![&string[..]];
        assert_eq!(
            exclusive_separate_at_positions(&string, positions),
            expected_result
        );

        let string = "aaasaa";
        let positions: Vec<usize> = vec![3];
        let expected_result = vec![&string[0..3], &string[4..]];
        assert_eq!(
            exclusive_separate_at_positions(&string, positions),
            expected_result
        );

        let string = "s";
        let positions: Vec<usize> = vec![0];
        let expected_result = vec![&string[0..0], &string[1..]];
        assert_eq!(
            exclusive_separate_at_positions(&string, positions),
            expected_result
        );
    }
}
