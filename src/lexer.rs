#[derive(PartialEq, Debug)]
enum Token<'a> {
    Progname(&'a str),
    Argument(&'a str),
    Separator,
    Quit,
}

#[derive(PartialEq, Debug)]
enum ParseError<'a> {
    UnbalancedQuote(&'a str),
}

fn tokenize_string(string: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    tokens
}

fn tokenize_atomic_string(string: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    Ok(tokens)
}

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

fn find_quoted_block_position(string: &str) -> Result<Option<(usize, usize)>, ParseError> {
    let first_occurrence = string.find('\"');
    let last_occurrence = string.rfind('\"');
    match (first_occurrence, last_occurrence) {
        (None, _) => Ok(None),
        (_, None) => Ok(None),
        (Some(fb), Some(lb)) => {
            if has_balanced_apostrophes(&string[fb..(lb+1)], '\"') {
                Ok(Some((fb, lb)))
            } else {
                Err(ParseError::UnbalancedQuote(&string[fb..lb+1]))
            }
        },
    }
}

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
                has_balanced_apostrophes(&string[(fb+1)..(lb)], apostrophe_char)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn apostrophe() {
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
    fn separator_position() {
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
    fn exclusive_separator() {
        let string = "aaaaa";
        let mut positions: Vec<usize> = Vec::new();
        let expected_result = vec![&string[..]];
        assert_eq!(exclusive_separate_at_positions(&string, positions), expected_result);

        let string = "aaasaa";
        let mut positions: Vec<usize> = vec![3];
        let expected_result = vec![&string[0..3], &string[4..]];
        assert_eq!(exclusive_separate_at_positions(&string, positions), expected_result);

        let string = "s";
        let mut positions: Vec<usize> = vec![0];
        let expected_result = vec![&string[0..0], &string[1..]];
        assert_eq!(exclusive_separate_at_positions(&string, positions), expected_result);
    }

    #[test]
    fn quoted_block_position() {
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
}
