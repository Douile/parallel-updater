use crate::util::default_true;

/// Represents details needed to start a program.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Program {
    /// Path to executable of program
    pub exe: String,
    /// Optional arguments for the program
    pub argv: Option<Vec<String>>,
    /// Optional extra environment variables for the program
    pub environ: Option<Vec<(String, String)>>,
    /// The directory the program should be executed in
    pub working_directory: Option<String>,
    /// Whether to past through the host programs environment.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub passthrough_environ: bool,
}

impl Program {
    /// Create new Program (builder)
    pub fn new(exe: impl Into<String>) -> Self {
        Program {
            exe: exe.into(),
            argv: None,
            environ: None,
            working_directory: None,
            passthrough_environ: true,
        }
    }

    pub fn argv(mut self, argv: impl Into<Vec<String>>) -> Self {
        self.argv = Some(argv.into());

        self
    }

    pub fn argv_str(self, argv: &[&str]) -> Self {
        self.argv(argv.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    pub fn environ(mut self, environ: Vec<(String, String)>) -> Self {
        self.environ = Some(environ);

        self
    }

    pub fn working_directory(mut self, working_directory: String) -> Self {
        self.working_directory = Some(working_directory);

        self
    }

    pub fn passthrough_environ(mut self, passthough_environ: bool) -> Self {
        self.passthrough_environ = passthough_environ;

        self
    }
}
