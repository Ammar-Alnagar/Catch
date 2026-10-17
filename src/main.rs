mod cli;
mod config;
mod docs;
mod doctor;
mod generator;
mod lock;
mod runner;
mod templates;

use anyhow::Result;
use clap::Parser;

use cli::{
    cmd_add, cmd_build, cmd_clean, cmd_new, cmd_remove, cmd_run, cmd_test, cmd_update, Cli, Commands,
};
use docs::cmd_docs;
use doctor::cmd_doctor;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Some(Commands::New { name, cuda, mlir, force }) => {
            cmd_new(&name, cuda, mlir, force)?;
        }
        Some(Commands::Build { release }) => {
            cmd_build(release, verbose)?;
        }
        Some(Commands::Run { release, args }) => {
            cmd_run(release, verbose, args)?;
        }
        Some(Commands::Test { release }) => {
            cmd_test(release, verbose)?;
        }
        Some(Commands::Clean) => {
            cmd_clean()?;
        }
        Some(Commands::Add {
            url,
            tag,
            branch,
            rev,
            name,
            target,
        }) => {
            cmd_add(url, tag, branch, rev, name, target)?;
        }
        Some(Commands::Remove { name }) => {
            cmd_remove(&name)?;
        }
        Some(Commands::Update { name }) => {
            cmd_update(name.as_deref())?;
        }
        Some(Commands::Docs { stdout }) => {
            cmd_docs(stdout)?;
        }
        Some(Commands::Doctor) => {
            cmd_doctor()?;
        }
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help()?;
            println!();
        }
    }

    Ok(())
}
