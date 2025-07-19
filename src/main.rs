mod peer;
mod crypto;
mod storage;
mod ai;
mod api;
mod grpc;
mod models;
mod web;
mod cli;

use storage::Storage;
use std::sync::Arc;
use crate::peer::P2PNode;
use clap::{Parser, ArgAction};

#[derive(Parser)]
#[command(name = "dafs")]
#[command(about = "Decentralized AI File System")]
struct Cli {
    /// Start interactive CLI shell
    #[arg(long, short)]
    cli: bool,
    
    /// Start the web dashboard server
    #[arg(long, action = ArgAction::SetTrue)]
    web: bool,
    
    /// Start the HTTP API server
    #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
    api: bool,
    
    /// Start the gRPC server
    #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
    grpc: bool,
    
    /// Start the P2P network
    #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
    p2p: bool,
    
    /// Web dashboard port
    #[arg(long, default_value = "3093")]
    web_port: u16,
    
    /// HTTP API port
    #[arg(long, default_value = "6543")]
    api_port: u16,
    
    /// gRPC port
    #[arg(long, default_value = "50051")]
    grpc_port: u16,
    
    /// P2P port
    #[arg(long, default_value = "2093")]
    p2p_port: u16,
    
    /// Run in integrated mode (start all services)
    #[arg(long, action = ArgAction::SetTrue)]
    integrated: bool,
    
    /// CLI subcommands
    #[command(subcommand)]
    command: Option<cli::Commands>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    // If CLI command is provided, handle it
    if let Some(command) = cli.command {
        let rt = tokio::runtime::Runtime::new()?;
        let result = rt.block_on(cli::dispatch_command(command));
        return result.map_err(|e| anyhow::anyhow!(e));
    }
    
    // If --cli flag is provided, start interactive shell
    if cli.cli {
        cli::run_repl().await;
        return Ok(());
    }
    
    // If no specific services are requested, run in integrated mode
    let integrated_mode = cli.integrated || (!cli.web && !cli.api && !cli.grpc && !cli.p2p);
    
    if integrated_mode {
        println!("🚀 Starting DAFS node in integrated mode...");
        
        // Initialize storage
        let storage = Arc::new(Storage::new("dafs_db")?);

        // Initialize P2P node
        let p2p = Arc::new(P2PNode::new());

        // Start HTTP API server in background
        let api_storage = storage.clone();
        let api_p2p = p2p.clone();
        tokio::spawn(async move { 
            api::run_with_storage_and_p2p(api_storage, api_p2p).await 
        });

        // Start gRPC server in background
        let grpc_storage = storage.clone();
        let grpc_p2p = p2p.clone();
        tokio::spawn(async move {
            if let Err(e) = grpc::run_grpc_server(grpc_storage, grpc_p2p).await {
                eprintln!("gRPC server error: {}", e);
            }
        });

        // Note: Web dashboard is NOT started automatically in integrated mode
        // It must be started via CLI command

        println!("✅ DAFS node started in integrated mode!");
        println!("   HTTP API: http://127.0.0.1:{}", cli.api_port);
        println!("   gRPC: grpc://[::1]:{}", cli.grpc_port);
        println!("   Web Dashboard: Use 'dafs cli startweb' to start");
        println!("   Use Ctrl+C to stop");
    } else {
        println!("🚀 Starting DAFS services...");
        
        // Initialize storage
        let storage = Arc::new(Storage::new("dafs_db")?);

        // Initialize P2P node
        let p2p = Arc::new(P2PNode::new());

        // Start requested services
        if cli.api {
            let api_storage = storage.clone();
            let api_p2p = p2p.clone();
            tokio::spawn(async move { 
                api::run_with_storage_and_p2p(api_storage, api_p2p).await 
            });
            println!("✅ HTTP API server started on port {}", cli.api_port);
        }

        if cli.grpc {
            let grpc_storage = storage.clone();
            let grpc_p2p = p2p.clone();
            tokio::spawn(async move {
                if let Err(e) = grpc::run_grpc_server(grpc_storage, grpc_p2p).await {
                    eprintln!("gRPC server error: {}", e);
                }
            });
            println!("✅ gRPC server started on port {}", cli.grpc_port);
        }

        if cli.web {
            tokio::spawn(async move {
                if let Err(e) = web::run_web_server().await {
                    eprintln!("Web dashboard server error: {}", e);
                }
            });
            println!("✅ Web dashboard server started on port {}", cli.web_port);
        }

        if cli.p2p {
            println!("✅ P2P network started on port {}", cli.p2p_port);
        }

        println!("   Use Ctrl+C to stop");
    }

    // Keep main alive
    futures::future::pending::<()>().await;
    Ok(())
}
