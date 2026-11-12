//! Command-line interface: `fjord validate|build|export`.
//!
//! Exit codes: 0 on success (warnings allowed), 1 for a validation, input or
//! output failure, 2 for a usage error. Diagnostics and warnings go to
//! stderr; `PASS:` and `Wrote` lines go to stdout.

use std::collections::HashSet;
use std::env;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::builder::{PossibleValuesParser, TypedValueParser as _};
use clap::{Args, Parser, Subcommand};

use crate::assets;
use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule, Severity};
use crate::export::{self, BUILD_SUFFIX, Format};
use crate::policy::contract::Mode;
use crate::project::{Composition, Project, Selection, check_asset_bindings, validate_tokens};

const EXAMPLES: &str = "\
Examples:
  fjord validate
  fjord build --mode day --theme tokens/themes/fern.tokens.json
  fjord export --format css --mode aurora --override compact.tokens.json";

#[derive(Debug, Parser)]
#[command(
    name = "fjord",
    version,
    about = "Validate the Fjord design tokens and export them as DTCG, resolved JSON or CSS.",
    after_help = EXAMPLES,
    infer_long_args = false,
    infer_subcommands = false
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate every mode × theme composition and the pinned font and icon assets.
    #[command(after_help = "Example:\n  fjord validate --root path/to/fjord-design")]
    Validate {
        /// Project root; defaults to the nearest ancestor of the current
        /// directory containing tokens/contract.json.
        #[arg(long, value_name = "DIR")]
        root: Option<PathBuf>,
    },
    /// Write the validated DTCG document (default dist/fjord.<mode>.tokens.json).
    #[command(after_help = "Example:\n  fjord build --mode day --output dist/day.tokens.json")]
    Build(Request),
    /// Write one export format (default dist/fjord.<mode>.{dtcg.json,resolved.json,tokens.css}).
    #[command(
        after_help = "Example:\n  fjord export --format css --mode night --theme tokens/themes/fern.tokens.json"
    )]
    Export {
        /// Output format.
        #[arg(long, value_enum)]
        format: Format,
        #[command(flatten)]
        request: Request,
    },
}

#[derive(Debug, Args)]
struct Request {
    /// Project root; defaults to the nearest ancestor of the current
    /// directory containing tokens/contract.json.
    #[arg(long, value_name = "DIR")]
    root: Option<PathBuf>,
    /// Mode layer.
    #[arg(
        long,
        default_value = "night",
        value_parser = PossibleValuesParser::new(Mode::ALL.map(Mode::as_str)).try_map(|s| s.parse::<Mode>())
    )]
    mode: Mode,
    /// Theme layer replacing the contract's default theme.
    #[arg(long, value_name = "FILE")]
    theme: Option<PathBuf>,
    /// Override layer, applied after the theme; repeat to apply several from left to right.
    #[arg(long = "override", value_name = "FILE")]
    overrides: Vec<PathBuf>,
    /// Output file; inside the project only under dist/.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
}

/// Parse the process arguments, run the command and report its exit code.
#[must_use]
pub fn run() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            return ExitCode::from(u8::try_from(e.exit_code()).unwrap_or(2));
        }
    };
    let (outcome, trailer) = match cli.command {
        Command::Validate { root } => (validate(root), ""),
        Command::Build(request) => (build(request, None), "; nothing was written"),
        Command::Export { format, request } => {
            (build(request, Some(format)), "; nothing was written")
        }
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure(diags)) => {
            let mut stderr = io::stderr().lock();
            for d in &diags {
                let _ = writeln!(stderr, "{d}");
            }
            let errors = diags
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let _ = writeln!(stderr, "ERROR {errors} problem(s){trailer}");
            ExitCode::FAILURE
        }
    }
}

/// `--root`, else the nearest ancestor of the cwd containing `tokens/contract.json`.
fn project_root(root: Option<PathBuf>) -> Outcome<PathBuf> {
    if let Some(root) = root {
        return Ok(root);
    }
    let cwd = current_dir()?;
    cwd.ancestors()
        .find(|dir| dir.join("tokens").join("contract.json").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Failure::one(
                Diagnostic::error(
                    Rule::Io,
                    cwd.display().to_string(),
                    "no Fjord project (tokens/contract.json) found here or in any parent directory",
                )
                .with_help("run inside the project or pass --root DIR"),
            )
        })
}

fn current_dir() -> Outcome<PathBuf> {
    env::current_dir().map_err(|e| {
        Failure::one(Diagnostic::error(
            Rule::Io,
            ".",
            format!("cannot read the current directory: {e}"),
        ))
    })
}

fn validate(root: Option<PathBuf>) -> Outcome<()> {
    let root = project_root(root)?;
    let project = Project::open(&root)?;
    let themes = project.official_themes()?;
    // The three default compositions come first; the asset checks use them.
    let mut selections: Vec<Selection> = Mode::ALL
        .iter()
        .map(|&mode| project.default_selection(mode))
        .collect();
    for mode in Mode::ALL {
        for theme in &themes {
            selections.push(Selection {
                theme: Some(theme.clone()),
                ..project.default_selection(mode)
            });
        }
    }
    let (tokens, (fonts, icons)) = rayon::join(
        || validate_tokens(&project, &selections),
        || {
            rayon::join(
                || assets::check_fonts(project.root()),
                || assets::check_icons(project.root()),
            )
        },
    );
    // Token and asset checks are independent: report the violations of both.
    let mut diags = Vec::new();
    let compositions = collect(tokens, &mut diags);
    let fonts = collect(fonts, &mut diags);
    let icons = collect(icons, &mut diags);
    let (Some(compositions), Some(fonts), Some(icons)) = (compositions, fonts, icons) else {
        return Err(Failure(diags));
    };
    for c in compositions.iter().take(Mode::ALL.len()) {
        collect(check_asset_bindings(c, &fonts, &icons), &mut diags);
    }
    if !diags.is_empty() {
        return Err(Failure(diags));
    }
    print_warnings(compositions.iter().flat_map(Composition::warnings));
    let tokens = compositions.first().map_or(0, |c| c.tokens().count());
    let pairs: usize = compositions.iter().map(Composition::pair_count).sum();
    stdout(&format!(
        "PASS: {tokens} DTCG tokens per composition; {} modes × {} themes; {pairs} contrast pairs in sRGB and HEX; {} font faces and {} icons with pinned licences.",
        Mode::ALL.len(),
        themes.len() + 1,
        fonts.face_count(),
        icons.count(),
    ));
    Ok(())
}

fn build(request: Request, format: Option<Format>) -> Outcome<()> {
    let root = project_root(request.root)?;
    let project = Project::open(&root)?;
    let defaults: Vec<Selection> = Mode::ALL
        .iter()
        .map(|&mode| project.default_selection(mode))
        .collect();
    let selection = Selection {
        mode: request.mode,
        theme: request.theme,
        overrides: request.overrides,
    };
    // The default compositions must stay valid whatever is requested (CLI1);
    // their warnings are not the request's and are not printed.
    let (preflight, composition) = rayon::join(
        || validate_tokens(&project, &defaults),
        || project.compose(&selection),
    );
    preflight?;
    let composition = composition?;
    print_warnings(composition.warnings().iter());
    let bytes = export::render(&composition, format.unwrap_or(Format::Dtcg))?;
    let (output, shown) = if let Some(output) = request.output {
        let shown = output.display().to_string();
        (output, shown)
    } else {
        let suffix = format.map_or(BUILD_SUFFIX, Format::suffix);
        let output = root.join(export::default_output(request.mode, suffix));
        let cwd = current_dir()?;
        let shown = output
            .strip_prefix(&cwd)
            .unwrap_or(&output)
            .display()
            .to_string();
        (output, shown)
    };
    export::write(&root, &output, &bytes, &composition.layer_files())?;
    stdout(&format!("Wrote {}: {shown}", request.mode.as_str()));
    Ok(())
}
/// The value of `outcome`, or `None` after moving its diagnostics into `diags`.
fn collect<T>(outcome: Outcome<T>, diags: &mut Vec<Diagnostic>) -> Option<T> {
    match outcome {
        Ok(value) => Some(value),
        Err(failure) => {
            diags.extend(failure.0);
            None
        }
    }
}

/// Print each warning once, keyed by rule, location and message.
fn print_warnings<'a>(warnings: impl Iterator<Item = &'a Diagnostic>) {
    let mut seen = HashSet::new();
    let mut stderr = io::stderr().lock();
    for w in warnings {
        if seen.insert((w.rule, w.location.as_str(), w.message.as_str())) {
            let _ = writeln!(stderr, "{w}");
        }
    }
}

fn stdout(line: &str) {
    let _ = writeln!(io::stdout().lock(), "{line}");
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("fjord").chain(args.iter().copied()))
    }

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn cli2_abbreviated_flags_are_usage_errors() {
        let e = parse(&["build", "--mod", "day"]).unwrap_err();
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn cli2_unknown_mode_and_missing_format_are_usage_errors() {
        assert_eq!(
            parse(&["build", "--mode", "dusk"]).unwrap_err().exit_code(),
            2
        );
        assert_eq!(parse(&["export"]).unwrap_err().exit_code(), 2);
        assert_eq!(parse(&["--help"]).unwrap_err().exit_code(), 0);
    }

    #[test]
    fn overrides_keep_command_line_order_and_mode_defaults_to_night() {
        let cli = parse(&[
            "export",
            "--format",
            "css",
            "--override",
            "b.json",
            "--override",
            "a.json",
        ])
        .unwrap();
        let Command::Export { format, request } = cli.command else {
            panic!("not export")
        };
        assert_eq!(format, Format::Css);
        assert_eq!(request.mode, Mode::Night);
        assert_eq!(
            request.overrides,
            [PathBuf::from("b.json"), PathBuf::from("a.json")]
        );
    }
}
