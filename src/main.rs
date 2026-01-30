mod replication;
mod decoder;
mod output;

use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "pgoutput-cmdline")]
#[command(about = "Stream PostgreSQL logical replication changes to stdout", long_about = None)]
struct Args {
    /// PostgreSQL connection string (e.g., "host=localhost user=postgres dbname=mydb")
    #[arg(short, long)]
    connection: String,

    /// Replication slot name
    #[arg(short, long)]
    slot: String,

    /// Publication name
    #[arg(short, long)]
    publication: String,

    /// Output format: json, json-pretty, or text
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Create replication slot if it doesn't exist
    #[arg(long)]
    create_slot: bool,

    /// Starting LSN (Log Sequence Number) to stream from
    #[arg(long)]
    start_lsn: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    eprintln!("Connecting to PostgreSQL...");
    eprintln!("Slot: {}", args.slot);
    eprintln!("Publication: {}", args.publication);
    eprintln!("Output format: {}", args.format);

    // Initialize replication stream
    let mut stream = replication::ReplicationStream::new(
        &args.connection,
        &args.slot,
        &args.publication,
        args.create_slot,
        args.start_lsn,
    )
    .await?;

    eprintln!("Starting replication stream...\n");

    // Set up graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        eprintln!("\nReceived shutdown signal, stopping...");
        let _ = shutdown_tx.send(()).await;
    });

    // Process replication stream
    let output_format = output::OutputFormat::from_str(&args.format)?;
    
    loop {
        tokio::select! {
            result = stream.next_message() => {
                match result {
                    Ok(Some(change)) => {
                        output::print_change(&change, &output_format)?;
                    }
                    Ok(None) => {
                        // Keep-alive or no data
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error reading replication stream: {}", e);
                        return Err(e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                eprintln!("Shutting down gracefully...");
                break;
            }
        }
    }

    Ok(())
}
