#[macro_use]
extern crate rust_i18n;

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use clap::{arg, Parser};
use log::debug;
use rust_i18n::i18n;
use serde::{Deserialize, Serialize};
use teloxide::adaptors::Throttle;
use teloxide::adaptors::throttle::Limits;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use telegram2photoprism::{PhotoPrismPhotoService, PhotoUID};
use telegram2photoprism::PhotoService;

i18n!("resources/locales", fallback = "en");

const UNKNOWN_EXTENSION: &str = "unknown";

#[derive(Serialize, Deserialize)]
struct TagKeyboardData {
    id: i32,
    values: Vec<i32>,
    photo_uid: String,
}

impl TagKeyboardData {
    const SAVE_BUTTON_ID: i32 = -2;

    const CHECK_MARK_SYMBOL: char = '\u{2713}';
}

type Bot = Throttle<teloxide::Bot>;

/// telegram2photoprism is a bot that downloads images and videos from a Telegram channel and uploads them to a PhotoPrism server.
/// You can also add tags to your images and videos.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// telegram bot access token
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_TELEGRAM_ACCESS_TOKEN")]
    telegram_access_token: String,
    /// telegram chat id from where photo will be downloaded and uploaded to PhotoPrism server.
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_TELEGRAM_CHAT_ID")]
    telegram_chat_id: i64,
    /// Telegram bot API server. For more information, visit https://github.com/tdlib/telegram-bot-api
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_BOT_API_SERVER", default_value = "https://api.telegram.org")]
    telegram_bot_api_server: String,
    /// Tags from which the user will choose tags for the photo.
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_TAGS", value_delimiter = ',')]
    tags: Vec<String>,
    /// PhotoPrism URL
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_PHOTOPRISM_URL")]
    photoprism_url: String,
    /// PhotoPrism username
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_PHOTOPRISM_USERNAME")]
    photoprism_username: String,
    /// PhotoPrism password
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_PHOTOPRISM_PASSWORD")]
    photoprism_password: String,
    /// Number of seconds after which the bot should obtain a new X-SESSION-ID using the username and password.
    /// Should be less than PHOTOPRISM_SESSION_TIMEOUT (https://docs.photoprism.app/getting-started/config-options/)
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_PHOTOPRISM_SESSION_REFRESH_SEC", default_value_t = 86400)]
    photoprism_session_refresh_sec: u64,
    /// Locale
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_LOCALE", value_parser = ["en", "ru"], default_value = "en")]
    locale: String,
    /// Directory for temporary downloaded files
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_WORKING_DIR", default_value_os_t = Self::get_default_working_dir())]
    working_dir: PathBuf,
    /// By default, Telegram compresses videos and images if they are not attached as files.
    /// The quality of the files is significantly reduced after compression.
    /// This option prohibits the bot from uploading compressed files to the PhotoPrism server.
    #[arg(long, env = "TELEGRAM2PHOTOPRISM_DISALLOW_COMPRESSED_FILES", default_value_t = false)]
    disallow_compressed_files: bool,
}

impl Args {
    fn get_default_working_dir() -> PathBuf {
        env::current_dir().unwrap()
    }
}

struct ApplicationContext {
    working_dir: OsString,
    disallow_compressed_files: bool,
    tags: Vec<String>,
}

impl ApplicationContext {
    pub fn new(args: &Args) -> Self {
        Self {
            tags: args.tags.clone(),
            working_dir: args.working_dir.clone().into_os_string(),
            disallow_compressed_files: args.disallow_compressed_files,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();
    let args = Args::parse();

    rust_i18n::set_locale(&args.locale);
    let context = ApplicationContext::new(&args);
    let chat_id = ChatId(args.telegram_chat_id);
    let bot: Bot = teloxide::Bot::new(args.telegram_access_token)
        .set_api_url(reqwest::Url::parse(&args.telegram_bot_api_server)?)
        .throttle(Limits::default());

    let photoservice = PhotoPrismPhotoService::new(
        args.photoprism_url,
        args.photoprism_username,
        args.photoprism_password,
        args.photoprism_session_refresh_sec,
    );

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter(move |m: Message| { m.chat.id == chat_id })
                .endpoint(handle_media_message_with_error)
        )
        .branch(
            Update::filter_callback_query()
                .filter(move |callback: CallbackQuery| {
                    match callback.message {
                        Some(msg) => msg.chat.id == chat_id,
                        None => false
                    }
                })
                .endpoint(handle_callback_message_with_error)
        );


    // Create a dispatcher for our bot
    Dispatcher::builder(bot, handler)
        .distribution_function(|x| Some(x.id))
        .dependencies(dptree::deps![Arc::new(context), Arc::new(photoservice)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}


async fn handle_media_message(bot: &Bot, app_context: Arc<ApplicationContext>, photoservice: Arc<PhotoPrismPhotoService>, m: Message) -> Result<(), anyhow::Error> {
    if m.document().is_none() && app_context.disallow_compressed_files {
        bot
            .send_message(m.chat.id, t!("error-attach-media-as-document"))
            .reply_to_message_id(m.id)
            .await?;
        Ok(())
    } else {
        let file_id_opt = get_file_id(&m);
        let upload_started_message = bot
            .send_message(m.chat.id, t!("upload-started"))
            .reply_to_message_id(m.id)
            .await?;
        match file_id_opt {
            Some(file_id) => {
                debug!("file_id: {}", file_id);
                let downloaded_file_path =
                    download_file(bot, file_id, &app_context.working_dir).await?;
                let photo_uid_result =
                    photoservice.upload_photo(&downloaded_file_path).await;
                tokio::fs::remove_file(downloaded_file_path).await?;
                let photo_uid = photo_uid_result?;
                let tags_keyboard = make_tags_keyboard(
                    &TagKeyboardData { id: -1, values: vec![], photo_uid: photo_uid.0 },
                    &app_context.tags,
                );
                bot
                    .edit_message_text(upload_started_message.chat.id, upload_started_message.id, t!("success-file-is-uploaded"))
                    .reply_markup(tags_keyboard)
                    .await?;
                Ok(())
            }
            None => Err(anyhow!("File from message has not been found."))
        }
    }
}

// TODO: Need to add progress bar.
async fn download_file(bot: &Bot, file_id: String, working_dir: &OsString) -> Result<PathBuf, anyhow::Error> {
    let file = bot.get_file(file_id).await?;
    let path = Path::new(&file.path);
    if path.is_absolute() {
        /* Telegram bot api server is used in local mode.
        Do not need to download anything because file will be downloaded
        on bot.get_file call by telegram bot api server. */
        Ok(path.to_path_buf())
    } else {
        let file_extension = Path::new(&file.path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(UNKNOWN_EXTENSION);
        let mut path_buf = PathBuf::new();
        path_buf.push(working_dir);
        path_buf.push(&file.id);
        path_buf.set_extension(file_extension);
        let mut f = tokio::fs::File::create(&path_buf).await?;
        bot.download_file(&file.path, &mut f).await?;
        Ok(path_buf)
    }
}

async fn handle_callback_message(bot: &Bot, app_context: Arc<ApplicationContext>, photoservice: Arc<PhotoPrismPhotoService>, q: CallbackQuery) -> Result<(), anyhow::Error> {
    if let (Some(keyboard_data_str), Some(Message { id, chat, .. })) = (q.data, q.message) {
        let tag_keyboard_data: TagKeyboardData = serde_json::from_str(&keyboard_data_str)?;
        let new_keyboard = make_tags_keyboard(&tag_keyboard_data, &app_context.tags);
        if tag_keyboard_data.id == TagKeyboardData::SAVE_BUTTON_ID {
            let mut selected_tags: Vec<String> = Vec::new();
            for tag_id in tag_keyboard_data.values {
                selected_tags.push(app_context.tags[tag_id as usize].to_owned());
            }
            let photo_uid = &PhotoUID(tag_keyboard_data.photo_uid.to_owned());
            for tag in &selected_tags {
                photoservice.add_label(photo_uid, tag).await?;
            }

            let message = t!("success-save-tags", tags = &selected_tags.join(","));
            bot.edit_message_text(chat.id, id, message).await?;
        } else {
            bot
                .edit_message_reply_markup(chat.id, id)
                .reply_markup(new_keyboard)
                .await?;
        }
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

fn get_file_id(message: &Message) -> Option<String> {
    let maybe_photo_file_id: Option<String> = message
        .photo()
        .and_then(|photo_sizes| { photo_sizes.last() })
        .map(|photo_size| { photo_size.file.id.clone() });
    let maybe_video_file_id: Option<String> =
        message.video().map(|v| { v.file.id.to_owned() });
    // TODO: Add filter by extension
    let maybe_document_file_id: Option<String> =
        message.document().map(|d| {
            d.file.id.clone()
        });
    maybe_document_file_id.or(maybe_photo_file_id).or(maybe_video_file_id)
}

fn make_tags_keyboard(data: &TagKeyboardData, tags: &[String]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let mut new_values = data.values.to_owned();
    let element_pos = data.values.iter().position(|&x| x == data.id);
    if let Some(pos) = element_pos {
        new_values.remove(pos);
    } else if data.id >= 0 {
        new_values.push(data.id)
    }

    for (chunk_index, tags) in tags.chunks(3).enumerate() {
        let row = tags
            .iter()
            .enumerate()
            .map(|(index, tag)| {
                let global_index = (chunk_index * 3 + index) as i32;
                let callback_data = TagKeyboardData {
                    id: global_index,
                    values: new_values.to_owned(),
                    photo_uid: data.photo_uid.to_owned(),
                };
                let callback_data_as_string = serde_json::to_string(&callback_data).unwrap();
                if new_values.contains(&global_index) {
                    InlineKeyboardButton::callback(
                        format!("{}{}", tag, TagKeyboardData::CHECK_MARK_SYMBOL),
                        callback_data_as_string,
                    )
                } else {
                    InlineKeyboardButton::callback(tag, callback_data_as_string)
                }
            })
            .collect();
        keyboard.push(row);
    }

    let callback_data = TagKeyboardData {
        id: TagKeyboardData::SAVE_BUTTON_ID,
        values: new_values.to_owned(),
        photo_uid: data.photo_uid.to_owned(),
    };
    let callback_data_as_string = serde_json::to_string(&callback_data).unwrap();

    keyboard.push(vec![InlineKeyboardButton::callback(t!("save"), callback_data_as_string)]);
    InlineKeyboardMarkup::new(keyboard)
}

// TODO: Can error handling with dependencies be done more elegantly?
async fn handle_media_message_with_error(bot: Bot, app_context: Arc<ApplicationContext>, photoservice: Arc<PhotoPrismPhotoService>, m: Message) -> Result<(), anyhow::Error> {
    let chat_id = m.chat.id;
    let message_id = m.id;
    match handle_media_message(&bot, app_context, photoservice, m).await {
        Ok(()) => Ok(()),
        Err(err) => {
            bot
                .send_message(chat_id, t!("error-panic"))
                .reply_to_message_id(message_id)
                .await?;
            Err(err)
        }
    }
}

async fn handle_callback_message_with_error(bot: Bot, app_context: Arc<ApplicationContext>, photoservice: Arc<PhotoPrismPhotoService>, q: CallbackQuery) -> Result<(), anyhow::Error> {
    let chat_id_opt = q.message.as_ref().map(|x| x.chat.id);
    let message_id_opt = q.message.as_ref().map(|x| x.id);
    if let (Some(chat_id), Some(message_id)) = (chat_id_opt, message_id_opt) {
        match handle_callback_message(&bot, app_context, photoservice, q).await {
            Ok(()) => Ok(()),
            Err(err) => {
                bot.edit_message_text(chat_id, message_id, t!("error-panic")).await?;
                Err(err)
            }
        }
    } else {
        Ok(())
    }
}