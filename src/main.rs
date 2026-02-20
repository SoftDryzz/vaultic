mod adapters;
mod cli;
mod config;
mod core;

use clap::Parser;

use cli::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    // For commands that expect a single env, use the first --env value
    let single_env = args.env.first().map(|s| s.as_str());

    let result = match &args.command {
        Commands::Init => cli::commands::init::execute(args.verbose),
        Commands::Encrypt { file } => {
            cli::commands::encrypt::execute(file.as_deref(), single_env, &args.cipher)
        }
        Commands::Decrypt { file } => {
            cli::commands::decrypt::execute(file.as_deref(), single_env, &args.cipher)
        }
        Commands::Check => cli::commands::check::execute(),
        Commands::Diff { file1, file2 } => cli::commands::diff::execute(
            file1.as_deref(),
            file2.as_deref(),
            &args.env,
            &args.cipher,
        ),
        Commands::Resolve => cli::commands::resolve::execute(single_env, &args.cipher),
        Commands::Keys { action } => cli::commands::keys::execute(action),
        Commands::Log {
            author,
            since,
            last,
        } => cli::commands::log::execute(author.as_deref(), since.as_deref(), *last),
        Commands::Status => cli::commands::status::execute(),
        Commands::Hook { action } => cli::commands::hook::execute(action),
    };

    if let Err(e) = result {
        cli::output::error(&format!("Error: {e}"));
        std::process::exit(1);
    }
}
