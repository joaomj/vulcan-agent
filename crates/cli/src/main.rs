use clap::Parser;
use vulcan_server::config::ServerConfig;
use vulcan_server::server;

#[derive(Parser)]
#[command(
    name = "vulcan-agent",
    version,
    about = "A local-first Rust coding agent"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Start the server
    Serve {
        /// Path to the project to work on
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve { project: _ } => {
            let config = ServerConfig::load()?;
            server::serve(config).await?;
        }
    }

    Ok(())
}
