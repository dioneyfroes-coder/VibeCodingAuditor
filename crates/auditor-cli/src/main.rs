use clap::Parser;

/// Auditor local de arquitetura de projetos.
#[derive(Parser, Debug)]
#[command(name = "vibe-coding-auditor", version, about, long_about = None)]
struct Args;

fn main() {
    let _args = Args::parse();
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;

    use super::Args;

    #[test]
    fn help_flag_is_recognized() {
        let error = Args::try_parse_from(["vibe-coding-auditor", "--help"])
            .expect_err("--help must be handled by clap");
        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn version_flag_is_recognized() {
        let error = Args::try_parse_from(["vibe-coding-auditor", "--version"])
            .expect_err("--version must be handled by clap");
        assert_eq!(error.kind(), ErrorKind::DisplayVersion);
    }
}
