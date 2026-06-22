#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub command: Option<Command>,
    pub modules: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    List,
    Public,
}

impl Cli {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let args = args.into_iter().collect::<Vec<_>>();

        if let Some(command) = args.first().and_then(|arg| match arg.as_str() {
            "list" => Some(Command::List),
            "public" => Some(Command::Public),
            _ => None,
        }) {
            return Self {
                command: Some(command),
                modules: Vec::new(),
            };
        }

        Self {
            command: None,
            modules: args,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_fetch() {
        assert_eq!(
            Cli::parse([]),
            Cli {
                command: None,
                modules: Vec::new()
            }
        );
    }

    #[test]
    fn parses_list_command() {
        assert_eq!(
            Cli::parse(["list".to_string()]),
            Cli {
                command: Some(Command::List),
                modules: Vec::new()
            }
        );
    }

    #[test]
    fn parses_public_command() {
        assert_eq!(
            Cli::parse(["public".to_string()]),
            Cli {
                command: Some(Command::Public),
                modules: Vec::new()
            }
        );
    }

    #[test]
    fn parses_module_list() {
        assert_eq!(
            Cli::parse(["cpu".to_string(), "gpu".to_string(), "memory".to_string()]),
            Cli {
                command: None,
                modules: vec!["cpu".to_string(), "gpu".to_string(), "memory".to_string()]
            }
        );
    }
}
