mod bot;

use anyhow;
use clap::Parser;
use std::{fs, path, process};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true, help = "URL of the bot's homeserver")]
    homeserver: String,

    #[arg(required = true, help = "username of the bot account")]
    username: String,

    #[arg(required = false, help = "password of the bot account")]
    password: Option<String>,

    #[arg(
        required = false,
        short,
        long,
        default_value = "localhost",
        help = "address of the server running an ollama API"
    )]
    address: String,

    #[arg(
        required = false,
        short,
        long,
        default_value_t = 11434,
        help = "ollama API port"
    )]
    port: u16,

    #[arg(
        required = false,
        short = 'P',
        long = "password-file",
        help = "path to password file"
    )]
    password_file: Option<path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let api = format!("http://{}:{}/api/generate", args.address, args.port);
    let password = match args.password {
        Some(p) => p,
        None => match args.password_file {
            Some(pf) => fs::read_to_string(pf)?,
            None => {
                eprintln!("Either a password file or password is required.");
                process::exit(1);
            }
        },
    };

    bot::login_and_sync(&args.homeserver, &args.username, &password, api).await?;

    Ok(())
}
