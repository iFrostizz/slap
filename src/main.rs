use clap::Parser;
use detectors::{ai_sec::AIDetector, structs::StructsDetector, Detector, Detectors};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

mod cli;
mod detectors;
mod lsp;

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt().init();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    log::debug!("Starting slap");

    let args = cli::Args::parse();
    let path = std::env::current_dir().expect("failed to get getcwd");
    let server = lsp::SlapServer::new(path, args.transport);
    // let detectors: Vec<Box<dyn Detector>> = vec![Box::new(AIDetector), Box::new(U256Detector)];
    let detectors: Vec<Box<dyn Detector>> = vec![Box::new(StructsDetector)];
    let detectors = Detectors(detectors);
    server.serve(detectors).await;
}
