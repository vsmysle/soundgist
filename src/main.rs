use teloxide::prelude::*;
use teloxide::types::{MediaKind, MessageKind};

use teloxide::net::Download;

async fn handle_message(
    bot: Bot,
    msg: Message,
) -> anyhow::Result<()> {
    if let MessageKind::Common(common_msg) = msg.kind { match common_msg.media_kind {
        MediaKind::Voice(audio) => {
            process_audio(bot, msg.chat.id, &audio.voice.file.id).await?;
        }
        MediaKind::Audio(audio) => {
            process_audio(bot, msg.chat.id, &audio.audio.file.id).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please send a voice message or an audio file.")
                .await?;
        }
    } }
    Ok(())
}


async fn process_audio(bot: Bot, chat_id: ChatId, file_id: &str) -> anyhow::Result<()> {
    // 1. Download the audio file
    let file = bot.get_file(file_id).await?;

    // Create temporary file
    let mut buf = vec![];

    // Save audio file
    bot.download_file(&file.path, &mut buf).await?;

    // 2. Convert audio to text
    let text = convert_audio_to_text(buf).await?;

    // 3. Summarize the text
    let summary = summarize_text(text).await?;

    // 4. Send the summary back to the user
    bot.send_message(chat_id, summary).await?;

    Ok(())
}

async fn convert_audio_to_text(audio_data: Vec<u8>) -> anyhow::Result<String> {
    // Implement STT logic here
    // This function should be able to handle both voice messages and audio files
    let client = async_openai::Client::new();

    // Convert audio data to AudioInput
    let audio = async_openai::types::AudioInput::from_vec_u8(
        "input.mp3".to_string(),
        audio_data
    );

    let request = async_openai::types::CreateTranscriptionRequestArgs::default()
        .file(audio)
        .model("whisper-1")
        .build()?;

    let response = client.audio().transcribe_raw(request).await?;
    
    println!("translate_srt:");
    println!("{}", String::from_utf8_lossy(response.as_ref()));

    let transcription_text = String::from_utf8_lossy(response.as_ref());
    Ok(transcription_text.to_string())
}

async fn summarize_text(text: String) -> anyhow::Result<String> {
    let client = async_openai::Client::new();

    let messages = vec![
        async_openai::types::ChatCompletionRequestSystemMessage {
            content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                "You are a helpful assistant that summarizes text concisely.".into()
            ),
            name: None,
        }.into(),
        async_openai::types::ChatCompletionRequestUserMessage {
            content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                format!("Please summarize the following text:\n\n{}", text)
            ),
            name: None,
        }.into(),
    ];

    let request = async_openai::types::CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages(messages)
        .build()?;

    let response = client.chat().create(request).await?;
    
    Ok(response.choices[0].message.content.clone().unwrap_or_default())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    
    tracing::info!("Starting bot...");
    
    let bot = Bot::from_env();
    tracing::info!("Bot initialized successfully");
 
    let handler = Update::filter_message()
        .filter(|msg: Message| {
            msg.voice().is_some() || msg.audio().is_some()
        })
        .endpoint(handle_message);

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
