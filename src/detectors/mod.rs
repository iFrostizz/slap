use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};
use tower_lsp::lsp_types::Diagnostic;

pub mod ai_sec;
pub mod structs;

#[derive(Debug)]
pub enum LspMessage {
    Diagnostics {
        path: PathBuf,
        diags: Vec<Diagnostic>,
    },
    Error,
}

pub trait Detector: Sync + Send {
    fn run(
        &self,
        file: PathBuf,
        content: String,
    ) -> Pin<Box<dyn Future<Output = Vec<LspMessage>> + Send + '_>>;
}

// #[derive(Debug)]
// pub struct Detectors(Vec<AIDetector>);

pub struct Detectors(pub Vec<Box<dyn Detector>>);

impl std::fmt::Debug for Detectors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo!()
        Ok(())
    }
}

impl Detectors {
    pub async fn run(&self, path: &Path, content: String) -> Vec<LspMessage> {
        let mut messages = Vec::new();
        for detector in &self.0 {
            // messages.append(&mut detector.run(&path.clone(), &content).await);
            // messages.append(&mut detector.run().await);
            messages.append(&mut detector.run(path.to_owned(), content.clone()).await);
        }
        messages
    }
}
