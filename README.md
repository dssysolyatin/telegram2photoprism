# telegram2photoprism

![CI](https://github.com/dssysolyatin/telegram2photoprism/actions/workflows/ci.yml/badge.svg)

telegram2photoprism is a bot that downloads images and videos from a Telegram channel and uploads them to a PhotoPrism
server. You can also add tags to your images and videos which is not possible to do
using [photoprism with WebDAV](https://docs.photoprism.app/user-guide/sync/webdav/).

# Why ?

Telegram is a very convenient way to communicate with your family, and chats with family often include many important
pictures. It is very convenient to upload these pictures to a private PhotoPrism server directly from the Telegram
channel.

Another issue is that PhotoPrism's face detection is not ideal, so tags often need to be added to photos manually. In
most photos, there are only a few specific people, so when a photo is uploaded, the desire is to choose them from
predefined tags.

# Demo

https://github.com/dssysolyatin/telegram2photoprism/assets/1016070/bfc8ee4c-99b7-4044-a904-105c730b1d6a

# Installation

### Create telegram bot

1. Open a session with [BotFather](https://telegram.me/BotFather)
2. Enter `/newbot`
3. Enter name of the bot. For example: `PhotoPrismSync`
4. Enter a username for the bot.
5. Copy token in the safe place (it is needed for `--telegram-access-token`)
6. Enter `/setprivacy`
7. Select your bot and enter `Disable`. This allows the bot to see all messages in the chat where you added it.
8. Add your bot to the chat from which you want to download photos and upload them to the PhotoPrism server.
9. Open a session with [BotFather](https://telegram.me/BotFather) again and enter `/setjoingroups`. Then,
   enter `Disable` so the bot cannot be added to any other groups.
10. Detect the chat ID of the chat from the previous step. You can do this, for example,
    using [IDBot](https://t.me/username_to_id_bot). The ID should look like: `-4148908551`. This is needed so that
    telegram2photoprism accepts only messages from the chat mentioned in step 8. (See `--telegram-chat-id`)

### Set up [telegram bot api server](https://github.com/tdlib/telegram-bot-api)

The default Telegram server https://api.telegram.org doesn't allow downloading files larger than [20MB](https://core.telegram.org/bots/faq#how-do-i-download-files) from chats.
If you only plan to use the bot for photos, this is okay.
But if you want to upload videos larger than 20MB to the PhotoPrism server, you need to run
the [Telegram Bot API server](https://github.com/tdlib/telegram-bot-api) in local mode.
Follow the instructions [here](https://github.com/tdlib/telegram-bot-api).

The local telegram bot api server should be specified using `--telegram-bot-api-server`

### Run telegram2photoprism

```
telegram2photoprism \
--telegram-access-token <TELEGRAM_ACCESS_TOKEN> \
--telegram-chat-id <TELEGRAM_CHAT_ID> \
--photoprism-url <PHOTOPRISM_URL> \
--photoprism-username <PHOTOPRISM_USERNAME> \
--photoprism-password <PHOTOPRISM_PASSWORD> 
```

If you want to use environment variables or specify tags, telegram bot api server then see next section.

### More options:

| Argument                                                                                   | Description                                                                                                                                                                                                                                      | Default                  |
|--------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------|
| --telegram-access-token (env: TELEGRAM2PHOTOPRISM_TELEGRAM_ACCESS_TOKEN)                   | Telegram bot access token                                                                                                                                                                                                                        | -                        |
| --telegram-chat-id (env: TELEGRAM2PHOTOPRISM_TELEGRAM_CHAT_ID)                             | Telegram chat id from where photo will be downloaded and uploaded to PhotoPrism server                                                                                                                                                           | -                        |
| --telegram-bot-api-server (env: TELEGRAM2PHOTOPRISM_BOT_API_SERVER)                        | Telegram bot API server. For more information, visit [here](https://github.com/tdlib/telegram-bot-api)                                                                                                                                           | https://api.telegram.org |
| --tags (env: TELEGRAM2PHOTOPRISM_TAGS)                                                     | Tags from which the user will choose tags for the photo                                                                                                                                                                                          | -                        |
| --photoprism-url (env: TELEGRAM2PHOTOPRISM_PHOTOPRISM_URL)                                 | PhotoPrism URL                                                                                                                                                                                                                                   | -                        |
| --photoprism-username (env: TELEGRAM2PHOTOPRISM_PHOTOPRISM_USERNAME)                       | PhotoPrism username                                                                                                                                                                                                                              | -                        |
| --photoprism-password (env: TELEGRAM2PHOTOPRISM_PHOTOPRISM_PASSWORD)                       | PhotoPrism password                                                                                                                                                                                                                              | -                        |
| --photoprism-session-refresh-sec (env: TELEGRAM2PHOTOPRISM_PHOTOPRISM_SESSION_REFRESH_SEC) | Number of seconds after which the bot should obtain a new X-Auth-Token using the username and password. Should be less than PHOTOPRISM_SESSION_TIMEOUT ([More Info](https://docs.photoprism.app/getting-started/config-options/))                | 86400                    |
| --locale (env: TELEGRAM2PHOTOPRISM_LOCALE)                                                 | Locale                                                                                                                                                                                                                                           | en                       |
| --working-dir (env: TELEGRAM2PHOTOPRISM_WORKING_DIR)                                       | Directory for temporary downloaded files                                                                                                                                                                                                         | CURRENT_DIR              |
| --disallow-compressed-files (env: TELEGRAM2PHOTOPRISM_DISALLOW_COMPRESSED_FILES)           | By default, Telegram compresses videos and images if they are not attached as files. The quality of the files is significantly reduced after compression. This option prohibits the bot from uploading compressed files to the PhotoPrism server | -                        |
| -h, --help                                                                                 | Print help                                                                                                                                                                                                                                       | -                        |
| -V, --version                                                                              | Print version                                                                                                                                                                                                                                    | -                        |

# Code quality

This is my first project using the Rust language.
There are likely pieces of code which can be improved.
If you have any suggestions, please create a pull request or issue.

# TODO:

1. Support OAuth. PhotoPrism OAuth is not production ready - https://github.com/photoprism/photoprism/issues/3943.
2. Add statistic base progress bar. The [Telegram Bot API server](https://github.com/tdlib/telegram-bot-api) does not
   provide information about file download progress (See https://github.com/tdlib/telegram-bot-api/issues/37).
   The workaround is to take statistics for the last N downloads and tries to calculate an approximation. 
