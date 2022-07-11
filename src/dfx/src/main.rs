use crate::config::{dfx_version, dfx_version_str};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::logger::{create_root_logger, LoggingMode};

use anyhow::Error;
use clap::Parser;
use lib::diagnosis::{diagnose, Diagnosis, NULL_DIAGNOSIS};
use semver::Version;
use std::path::PathBuf;

mod actors;
mod commands;
mod config;
mod lib;
mod util;

/// The DFINITY Executor.
#[derive(Parser)]
#[clap(name("dfx"), version = dfx_version_str())]
pub struct CliOpts {
    #[clap(long, short('v'), parse(from_occurrences))]
    verbose: u64,

    #[clap(long, short('q'), parse(from_occurrences))]
    quiet: u64,

    #[clap(long("log"), default_value("stderr"), possible_values(&["stderr", "tee", "file"]))]
    logmode: String,

    #[clap(long)]
    logfile: Option<String>,

    #[clap(long)]
    identity: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

fn is_warning_disabled(warning: &str) -> bool {
    // By default, warnings are all enabled.
    let env_warnings = std::env::var("DFX_WARNING").unwrap_or_else(|_| "".to_string());
    env_warnings
        .split(',')
        .filter(|w| w.starts_with('-'))
        .any(|w| w.chars().skip(1).collect::<String>().eq(warning))
}

/// In some cases, redirect the dfx execution to the proper version.
/// This will ALWAYS return None, OR WILL TERMINATE THE PROCESS. There is no Ok()
/// version of this (nor should there be).
///
/// Note: the right return type for communicating this would be [Option<!>], but since the
/// never type is experimental, we just assert on the calling site.
fn maybe_redirect_dfx(version: &Version) -> Option<()> {
    // Verify we're using the same version as the dfx.json, and if not just redirect the
    // call to the cache.
    if dfx_version() != version {
        // Show a warning to the user.
        if !is_warning_disabled("version_check") {
            eprintln!(
                concat!(
                    "Warning: The version of DFX used ({}) is different than the version ",
                    "being run ({}).\n",
                    "This might happen because your dfx.json specifies an older version, or ",
                    "DFX_VERSION is set in your environment.\n",
                    "We are forwarding the command line to the old version. To disable this ",
                    "warning, set the DFX_WARNING=-version_check environment variable.\n"
                ),
                version,
                dfx_version()
            );
        }

        match crate::config::cache::call_cached_dfx(version) {
            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("Error when trying to forward to project dfx:\n{:?}", e);
                eprintln!("Installed executable: {}", dfx_version());
                std::process::exit(1)
            }
        };
    }

    None
}

/// Setup a logger with the proper configuration, based on arguments.
/// Returns a topple of whether or not to have a progress bar, and a logger.
fn setup_logging(opts: &CliOpts) -> (bool, slog::Logger) {
    // Create a logger with our argument matches.
    let level = opts.verbose as i64 - opts.quiet as i64;

    let mode = match opts.logmode.as_str() {
        "tee" => LoggingMode::Tee(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        "file" => LoggingMode::File(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        _ => LoggingMode::Stderr,
    };

    // Only show the progress bar if the level is INFO or more.
    (level >= 0, create_root_logger(level, mode))
}

fn print_error_and_diagnosis(err: Error, error_diagnosis: Diagnosis) {
    let mut stderr = util::stderr_wrapper::stderr_wrapper();

    // print error/cause stack
    for (level, cause) in err.chain().enumerate() {
        if level == 0 {
            stderr
                .fg(term::color::RED)
                .expect("Failed to set stderr output color.");
            write!(stderr, "Error: ").expect("Failed to write to stderr.");
            stderr
                .reset()
                .expect("Failed to reset stderr output color.");

            writeln!(stderr, "{}", err).expect("Failed to write to stderr.");
            continue;
        }
        if level == 1 {
            stderr
                .fg(term::color::YELLOW)
                .expect("Failed to set stderr output color.");
            write!(stderr, "Caused by: ").expect("Failed to write to stderr.");
            stderr
                .reset()
                .expect("Failed to reset stderr output color.");

            writeln!(stderr, "{}", err).expect("Failed to write to stderr.");
        }
        eprintln!("{:width$}{}", "", cause, width = level * 2);
    }

    // print diagnosis
    if let Some(error_explanation) = error_diagnosis.0 {
        stderr
            .fg(term::color::YELLOW)
            .expect("Failed to set stderr output color.");
        writeln!(stderr, "Error explanation:").expect("Failed to write to stderr.");
        stderr
            .reset()
            .expect("Failed to reset stderr output color.");

        writeln!(stderr, "{}", error_explanation).expect("Failed to write to stderr.");
    }
    if let Some(action_suggestion) = error_diagnosis.1 {
        stderr
            .fg(term::color::YELLOW)
            .expect("Failed to set stderr output color.");
        writeln!(stderr, "How to resolve the error:").expect("Failed to write to stderr.");
        stderr
            .reset()
            .expect("Failed to reset stderr output color.");

        writeln!(stderr, "{}", action_suggestion).expect("Failed to write to stderr.");
    }
}

fn main() {
    let cli_opts = CliOpts::parse();
    let (progress_bar, log) = setup_logging(&cli_opts);
    let identity = cli_opts.identity;
    let command = cli_opts.command;
    let mut error_diagnosis: Diagnosis = NULL_DIAGNOSIS;
    let result = match EnvironmentImpl::new() {
        Ok(env) => {
            maybe_redirect_dfx(env.get_version()).map_or((), |_| unreachable!());
            match EnvironmentImpl::new().map(|env| {
                env.with_logger(log)
                    .with_progress_bar(progress_bar)
                    .with_identity_override(identity)
            }) {
                Ok(env) => {
                    slog::trace!(
                        env.get_logger(),
                        "Trace mode enabled. Lots of logs coming up."
                    );
                    match commands::exec(&env, command) {
                        Err(e) => {
                            error_diagnosis = diagnose(&env, &e);
                            Err(e)
                        }
                        ok => ok,
                    }
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    };
    if let Err(err) = result {
        print_error_and_diagnosis(err, error_diagnosis);
        std::process::exit(255);
    }
}