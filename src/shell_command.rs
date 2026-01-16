
pub struct ShellCommand {
    pub arguments: String,
    pub command: String
}

impl ShellCommand {
    pub fn parse(mut cmd: String) -> ShellCommand {
        cmd = cmd.replace("\n", "");
        if let Some(split) = cmd.split_once(" ") {
                return Self {
                    command: split.0.to_string(),
                    arguments:  split.1.to_string()
                };
        }

        return Self {
            command: cmd,
            arguments: "".to_string()
        };
    }
}
