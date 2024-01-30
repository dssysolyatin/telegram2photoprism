# telegram2photoprism

telegram2photoprism is a bot that downloads images and videos from a Telegram channel and uploads them to a PhotoPrism
server. You can also add tags to your images and videos which is not possible to do
using [photoprism with WebDAV](https://docs.photoprism.app/user-guide/sync/webdav/).

# Code quality

This is my first project using the Rust language.
There are likely pieces of code which can be improved.
If you have any suggestions, please create a pull request or issue.

# TODO:

1. Add documentation and video demo
2. Add statistic base progress bar. The [Telegram Bot API server](https://github.com/tdlib/telegram-bot-api) does not
   provide information about file download progress (See https://github.com/tdlib/telegram-bot-api/issues/37).
   The workaround is to take statistics for the last N downloads and tries to calculate an approximation. 