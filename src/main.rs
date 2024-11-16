use clap::Parser;

mod cli;
mod lsp;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let args = cli::Args::parse();
    lsp::SlapServer::serve(args.transport).await;
}
