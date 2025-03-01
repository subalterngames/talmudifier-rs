pub struct LatexCommand {
    /// Start the command, e.g. `\textit{`
    pub prefix: Option<&'static str>,
    /// End the command, e.g. `}`
    pub suffix: Option<&'static str>,
}
