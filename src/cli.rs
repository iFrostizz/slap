use clap::{Parser, ValueEnum};

#[derive(Debug, Default, ValueEnum, Clone)]
pub enum Transport {
    #[default]
    Stdio,
    #[allow(clippy::upper_case_acronyms)]
    IPC,
    #[allow(clippy::upper_case_acronyms)]
    TCP,
}

/// Slap: Solidity LSP
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Communicate over STDIO instead of TCP
    #[arg(long, default_value = "stdio")]
    pub transport: Transport,
}
