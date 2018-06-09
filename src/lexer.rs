enum Token<'a> {
    Progname(&'a str),
    Argument(&'a str),
    Separator,
    Quit,
}

fn tokenize_string(string: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    tokens
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

fn has_balanced_apostrophes(string: &str, apostrophe_char: char) -> bool {
    println!("{}", string);
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
}
