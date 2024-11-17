use super::{Detector, LspMessage};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::{future::Future, path::PathBuf, pin::Pin};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

#[derive(Debug)]
pub struct AIDetector;

impl Detector for AIDetector {
    fn run(
        &self,
        file: PathBuf,
        content: String,
    ) -> Pin<Box<dyn Future<Output = Vec<LspMessage>> + Send + '_>> {
        Box::pin(async move {
            // run some AI
            // todo!()

            let prompt = format!(
            "Here is a piece of Solidity code.
        Please report any improvement / vulnerability if any as a list at each line with the starting position (line, character) and ending position (line, character)
        followed by the text of the vulnerability. Not a paragraph.
        ```solidity
        {content}
        ```");
            // let prompt = "hello";

            let token = "sk-LVl6n9zaMmgLFQ3omYMIYVwuxBRUoRWtQRB4MgPQLaKEWQ8A";

            let body = format!(
                "{{
  \"model\": \"gpt-4o\",
  \"messages\": [
    {{\"role\": \"user\", \"content\": \"{}\"}}
  ]
}}",
                prompt
            );

            let body = body.replace('\n', "");

            let client = reqwest::Client::new();
            let res = client
                .post("https://api.red-pill.ai/v1/chat/completions")
                .header(CONTENT_TYPE, "application/json")
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .body(body)
                .send()
                .await;

            let res: Resp = res.unwrap().json().await.unwrap();

            res.choices
                .into_iter()
                .map(|choice| choice.message)
                .map(|message| LspMessage::Diagnostics {
                    path: file.to_owned(),
                    diags: vec![Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(1, 10)),
                        severity: Some(DiagnosticSeverity::HINT),
                        message: message.content,
                        ..Default::default()
                    }],
                })
                .collect()
        })
    }
}

// {
//   "id": "chatcmpl-AUEX5UZmBNGs2PeiJopTYiUqzCeyF",
//   "object": "chat.completion",
//   "created": 1731769067,
//   "model": "gpt-4o-2024-08-06",
//   "choices": [
//     {
//       "index": 0,
//       "message": {
//         "role": "assistant",
//         "content": "The meaning of life is a profound and often philosophical question that has been contemplated by thinkers, theologians, and individuals throughout history. Various perspectives offer different interpretations:\n\n1. **Philosophical Views**: Many philosophers have attempted to answer this question through different lenses. Existentialists, for example, may argue that life's meaning is not given but created by each individual through their choices and actions.\n\n2. **Religious Perspectives**: Most religions provide their own definitions of life's meaning, often related to spiritual fulfillment, adherence to divine commandments, or preparation for an afterlife. For example, in Christianity, life's purpose might involve following Jesus's teachings and seeking a relationship with God.\n\n3. **Scientific Views**: From a scientific standpoint, life's meaning might be linked to survival, reproduction, or the propagation of DNA. Some might argue that meaning is not inherent in life itself but is a construct of the human mind, born out of a need for purpose and order.\n\n4. **Personal Views**: On a personal level, individuals might find meaning through relationships, personal achievements, helping others, or pursuing passions and interests.\n\nUltimately, the meaning of life is a subjective question, and its answer can vary greatly depending on one's beliefs, values, and experiences.",
//         "refusal": null
//       },
//       "logprobs": null,
//       "finish_reason": "stop"
//     }
//   ],
//   "usage": {
//     "prompt_tokens": 14,
//     "completion_tokens": 253,
//     "total_tokens": 267,
//     "prompt_tokens_details": {
//       "cached_tokens": 0,
//       "audio_tokens": 0
//     },
//     "completion_tokens_details": {
//       "reasoning_tokens": 0,
//       "audio_tokens": 0,
//       "accepted_prediction_tokens": 0,
//       "rejected_prediction_tokens": 0
//     }
//   },
//   "system_fingerprint": "fp_159d8341cc"
// }

#[derive(Debug, Deserialize)]
struct Resp {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: u32,
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
}
