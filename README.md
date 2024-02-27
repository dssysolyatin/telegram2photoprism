# telegram2photoprism

![CI](https://github.com/dssysolyatin/telegram2photoprism/actions/workflows/ci.yml/badge.svg)

telegram2photoprism is a bot that downloads images and videos from a Telegram channel and uploads them to a PhotoPrism
server. You can also add tags to your images and videos which is not possible to do
using [photoprism with WebDAV](https://docs.photoprism.app/user-guide/sync/webdav/).

# Code quality

This is my first project using the Rust language.
There are likely pieces of code which can be improved.
If you have any suggestions, please create a pull request or issue.

# Documentation (In progress)

https://github.com/dssysolyatin/telegram2photoprism/assets/1016070/bfc8ee4c-99b7-4044-a904-105c730b1d6a

## Options:

| Argument                         | Environment Variable                               | Default Value            | Description                                                                                                                                                                  |
|----------------------------------|----------------------------------------------------|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| --telegram-access-token          | TELEGRAM2PHOTOPRISM_TELEGRAM_ACCESS_TOKEN          |                          | Telegram bot access token.                                                                                                                                                   |
| --telegram-chat-id               | TELEGRAM2PHOTOPRISM_TELEGRAM_CHAT_ID               |                          | Telegram chat ID from where photos will be downloaded.                                                                                                                       |
| --telegram-bot-api-server        | TELEGRAM2PHOTOPRISM_BOT_API_SERVER                 | https://api.telegram.org | Telegram bot API server.                                                                                                                                                     |
| --tags                           | TELEGRAM2PHOTOPRISM_TAGS                           |                          | Tags from which the user will choose tags for the photo.                                                                                                                     |
| --photoprism-url                 | TELEGRAM2PHOTOPRISM_PHOTOPRISM_URL                 |                          | PhotoPrism URL.                                                                                                                                                              |
| --photoprism-username            | TELEGRAM2PHOTOPRISM_PHOTOPRISM_USERNAME            |                          | PhotoPrism username.                                                                                                                                                         |
| --photoprism-password            | TELEGRAM2PHOTOPRISM_PHOTOPRISM_PASSWORD            |                          | PhotoPrism password.                                                                                                                                                         |
| --photoprism-session-refresh-sec | TELEGRAM2PHOTOPRISM_PHOTOPRISM_SESSION_REFRESH_SEC | 86400                    | Number of seconds after which the bot should obtain a new X-SESSION-ID using the username and password.                                                                      |
| --locale                         | TELEGRAM2PHOTOPRISM_LOCALE                         | en                       | Locale (possible values: en, ru).                                                                                                                                            |
| --working-dir                    | TELEGRAM2PHOTOPRISM_WORKING_DIR                    |                          | Directory for temporary downloaded files.                                                                                                                                    |
| --disallow-compressed-files      | TELEGRAM2PHOTOPRISM_DISALLOW_COMPRESSED_FILES      |                          | By default, Telegram compresses videos and images if they are not attached as files. This option prohibits the bot from uploading compressed files to the PhotoPrism server. |

# TODO:

1. Add documentation and video demo
2. Add statistic base progress bar. The [Telegram Bot API server](https://github.com/tdlib/telegram-bot-api) does not
   provide information about file download progress (See https://github.com/tdlib/telegram-bot-api/issues/37).
   The workaround is to take statistics for the last N downloads and tries to calculate an approximation. 
