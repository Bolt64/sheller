pub fn get_commands_from_line(line: &str) -> Vec<&str> {
    return line.split(';').map(|x| x.trim()).collect();
}

pub fn parse_command(command: &str) -> Vec<&str> {
    return command.split(';').map(|x| x.trim()).collect();
}
