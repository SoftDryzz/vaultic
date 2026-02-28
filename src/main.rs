mod adapters;
mod cli;
mod config;
mod core;

use clap::Parser;

use cli::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    // Initialize global CLI state before any command runs
    cli::output::init(args.verbose, args.quiet);
    cli::context::init(args.config.as_deref());

    // Passive version check (suppressed in quiet mode and during update)
    if !args.quiet
        && !matches!(args.command, Commands::Update)
        && let Some(latest) = adapters::updater::github_updater::check_latest_version()
    {
        cli::output::warning(&format!(
            "New version available: v{latest}. Run 'vaultic update' to upgrade."
        ));
    }

    // Validate all --env values before dispatching any command
    for env_name in &args.env {
        if let Err(e) = cli::context::validate_env_name(env_name) {
            cli::output::error(&format!("Error: {e}"));
            std::process::exit(1);
        }
    }

    // For commands that expect a single env, use the first --env value
    let single_env = args.env.first().map(|s| s.as_str());

    let result = match &args.command {
        Commands::Init => cli::commands::init::execute(),
        Commands::Encrypt { file, all } => {
            cli::commands::encrypt::execute(file.as_deref(), single_env, &args.cipher, *all)
        }
        Commands::Decrypt { file, key, output } => cli::commands::decrypt::execute(
            file.as_deref(),
            single_env,
            &args.cipher,
            key.as_deref(),
            output.as_deref(),
        ),
        Commands::Check => cli::commands::check::execute(),
        Commands::Diff { file1, file2 } => cli::commands::diff::execute(
            file1.as_deref(),
            file2.as_deref(),
            &args.env,
            &args.cipher,
        ),
        Commands::Resolve { output } => {
            cli::commands::resolve::execute(single_env, &args.cipher, output.as_deref())
        }
        Commands::Keys { action } => cli::commands::keys::execute(action),
        Commands::Log {
            author,
            since,
            last,
        } => cli::commands::log::execute(author.as_deref(), since.as_deref(), *last),
        Commands::Status => cli::commands::status::execute(),
        Commands::Hook { action } => cli::commands::hook::execute(action),
        Commands::Update => cli::commands::update::execute(),
    };

    if let Err(e) = result {
        cli::output::error(&format!("Error: {e}"));
        std::process::exit(1);
    }
}
