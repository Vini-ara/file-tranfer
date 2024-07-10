extern crate dotenv;

use dotenv::dotenv;

use clap::{Parser, Subcommand };
use file_transfer::{client::{SendingClient, FetchingClient}, server::Server};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Requests a file from the remote server.
    Get {
        /// Secret for the file request.
        #[clap(short, long)]
        secret: String,
    },
    /// Sends a file to the remote server.
    Send {
        #[clap(short, long)]
        file: String,
    },

    /// Runs the remote proxy server.
    Server {},
}

#[tokio::main]
async fn run(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Server {} => {
            let server = Server::new().await;

            server.listen().await;
        }
        Command::Send { file } => {
            let sending_client = SendingClient::new(file).await;

            sending_client.connect().await;
        }
        Command::Get { secret } => {
            let fetching_client = FetchingClient::new(secret).await;

            fetching_client.connect().await;
        }
    }

    Ok(())
}

fn main() {
    dotenv().ok();
    let _ = run(Args::parse().command);
}
