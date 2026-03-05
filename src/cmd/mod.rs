pub mod payment;
pub mod pricing;
pub mod query;
pub mod regsitry;

use clap::{Parser, Subcommand};

use crate::cmd::{
    payment::PaymentCommands, pricing::PricingCommands, query::QueryCommands,
    regsitry::RegistryCommands,
};

#[derive(Parser)]
#[command(name = "infrapass")]
#[command(about = "", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    // RPC URL
    #[arg(long, global = true)]
    pub rpc_url: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Provider management commands
    #[command(subcommand)]
    Provider(RegistryCommands),

    /// Pricing tier management commands
    #[command(subcommand)]
    Pricing(PricingCommands),

    /// Payment and settlement commands
    #[command(subcommand)]
    Payment(PaymentCommands),

    /// Query blockchain data
    #[command(subcommand)]
    Query(QueryCommands),
}
