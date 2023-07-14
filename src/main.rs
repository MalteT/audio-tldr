use std::path::Path;

use async_openai::{
    error::OpenAIError,
    types::{
        AudioInput, AudioResponseFormat, ChatCompletionRequestMessage,
        CreateChatCompletionResponse, CreateTranscriptionRequest, CreateTranscriptionResponse,
        Role,
    },
    Audio, Chat,
};
use lazy_static::lazy_static;
use teloxide::{
    net::Download,
    prelude::*,
    types::{ForwardedFrom, MediaAudio, MediaKind, MediaVoice, MessageCommon, MessageKind},
    RequestError,
};
use tokio::fs::File;
use uuid::Uuid;

use crate::tmp::TempFile;

mod tmp;

lazy_static! {
    static ref CLIENT: async_openai::Client<async_openai::config::OpenAIConfig> =
        async_openai::Client::new();
    static ref NO_RESPONSE: String = String::from("no response");
}

async fn download_telegram_file_by_id<P>(
    bot: &Bot,
    id: String,
    target: P,
) -> Result<(), RequestError>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let file = bot.get_file(id).await?;
    log::trace!("Downloading to {target:?}");
    let mut download_file = File::create(&target).await?;
    bot.download_file(&file.path, &mut download_file).await?;
    Ok(())
}

async fn transcribe_audio<P: AsRef<Path>>(
    source: P,
) -> Result<CreateTranscriptionResponse, OpenAIError> {
    Audio::new(&*CLIENT)
        .transcribe(CreateTranscriptionRequest {
            file: AudioInput::new(&source),
            model: "whisper-1".into(),
            response_format: Some(AudioResponseFormat::Json),
            temperature: Some(0.0),
            language: Some("de".into()),
            ..Default::default()
        })
        .await
}

async fn create_summary(
    text: String,
    username: &str,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    Chat::new(&*CLIENT)
        .create(async_openai::types::CreateChatCompletionRequest {
            model: "gpt-4".into(),
            messages: vec![ChatCompletionRequestMessage {
                role: Role::System,
                content: Some(format!("Generiere ein TL;DR der folgenden Nachricht ohne 'TL;DR' prefix. Die Nachricht is eine transkribierte Sprachnachricht von einer Person namens {username} welche in einem Chat geschickt wurde. Die Transkribtion kann fehlerhaft gewesen sein, korrigiere mögliche Fehler eigenständig")),
                name: None,
                function_call: None,
            }, ChatCompletionRequestMessage {
                role: Role::User,
                content: Some(text),
                name: None,
                function_call: None,
            }],
            n: Some(1),
            ..Default::default()
        })
        .await
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv::dotenv().log_err("Failed to load environment");
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, move |bot: Bot, msg: Message| async move {
        match msg {
            Message {
                kind:
                    MessageKind::Common(MessageCommon {
                        from,
                        media_kind: MediaKind::Voice(MediaVoice { voice, .. }),
                        forward,
                        ..
                    }),
                ..
            } => {
                let forward_origin = forward.and_then(|fw| match fw.from {
                    ForwardedFrom::User(user) => user.username,
                    ForwardedFrom::Chat(_) => None,
                    ForwardedFrom::SenderName(name) => Some(name),
                });
                let username = forward_origin
                    .or_else(|| from.and_then(|user| user.username))
                    .unwrap_or(String::from("Nutzer"));
                let download_target = TempFile::new(format!("{}.ogg", Uuid::new_v4()));
                download_telegram_file_by_id(&bot, voice.file.id, &download_target).await?;

                let transcription_result = transcribe_audio(&download_target)
                    .await
                    .log_err("Transcribing audio");

                match transcription_result {
                    Some(resp) => {
                        bot.send_message(msg.chat.id, &resp.text).await?;
                        let resp = create_summary(resp.text, &username)
                            .await
                            .log_err("TL;DR AI call failed");
                        match resp {
                            Some(resp) => {
                                bot.send_message(
                                    msg.chat.id,
                                    format!(
                                        "TL;DR: {}",
                                        resp.choices[0]
                                            .message
                                            .content
                                            .as_ref()
                                            .unwrap_or(&*NO_RESPONSE)
                                    ),
                                )
                                .await?;
                            }
                            None => {
                                bot.send_message(msg.chat.id, &*NO_RESPONSE).await?;
                            }
                        }
                    }
                    None => {
                        bot.send_message(msg.chat.id, "No transcription available")
                            .await?;
                    }
                }
                drop(download_target);
            }
            Message {
                kind:
                    MessageKind::Common(MessageCommon {
                        media_kind: MediaKind::Audio(MediaAudio { audio, .. }),
                        from,
                        forward,
                        ..
                    }),
                ..
            } => {
                let forward_origin = forward.and_then(|fw| match fw.from {
                    ForwardedFrom::User(user) => user.username,
                    ForwardedFrom::Chat(_) => None,
                    ForwardedFrom::SenderName(name) => Some(name),
                });
                let username = forward_origin
                    .or_else(|| from.and_then(|user| user.username))
                    .unwrap_or(String::from("Nutzer"));
                let download_target = TempFile::new(format!("{}.ogg", Uuid::new_v4()));
                download_telegram_file_by_id(&bot, audio.file.id, &download_target).await?;

                let transcription_result = transcribe_audio(&download_target)
                    .await
                    .log_err("Transcribing audio");

                match transcription_result {
                    Some(resp) => {
                        bot.send_message(msg.chat.id, &resp.text).await?;
                        let resp = create_summary(resp.text, &username)
                            .await
                            .log_err("TL;DR AI call failed");
                        match resp {
                            Some(resp) => {
                                bot.send_message(
                                    msg.chat.id,
                                    format!(
                                        "TL;DR: {}",
                                        resp.choices[0]
                                            .message
                                            .content
                                            .as_ref()
                                            .unwrap_or(&*NO_RESPONSE)
                                    ),
                                )
                                .await?;
                            }
                            None => todo!(),
                        }
                    }
                    None => {}
                }
                drop(download_target);
            }
            _ => {}
        }
        Ok(())
    })
    .await;
}

trait LogResult<T, E> {
    fn log_err<S: std::fmt::Display>(self, desc: S) -> Option<T>;
    fn log_warn<S: std::fmt::Display>(self, desc: S) -> Option<T>;
}

impl<T, E> LogResult<T, E> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn log_err<S: std::fmt::Display>(self, desc: S) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(why) => {
                log::error!("{desc}: {why}");
                None
            }
        }
    }
    fn log_warn<S: std::fmt::Display>(self, desc: S) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(why) => {
                log::warn!("{desc}: {why}");
                None
            }
        }
    }
}
