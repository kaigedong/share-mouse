use clap::{Parser, Subcommand};

/// Fetch real trading from chain, and read chance from log.
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Commands,

    /// Config path
    #[arg(short, long, default_value = "analysis_config.toml")]
    pub config_path: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start a server to share mouse and key
    Server {
        /// Which server addr to listen
        #[arg(short = 's', long, default_value = "0.0.0.0:8090")]
        server_listen: String,
    },
    /// Compare competitor's real transaction and our calculation
    Client {
        /// Connect which server
        #[arg(short = 'c', long)]
        connect_to: String,
    },
}
