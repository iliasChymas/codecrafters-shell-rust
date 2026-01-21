pub struct ShellCommand {
    pub arguments: String,
    pub command: String
}

pub fn parse(mut cmd: String) -> ShellCommand {
    cmd = cmd.replace("\n", "");
    if let Some(split) = cmd.split_once(" ") {
            return ShellCommand {
                command: split.0.to_string(),
                arguments:  split.1.to_string()
            };
    }

    return ShellCommand {
        command: cmd,
        arguments: "".to_string()
    };
}
