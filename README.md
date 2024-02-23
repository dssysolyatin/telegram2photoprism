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

# TODO:

1. Add documentation and video demo
2. Add statistic base progress bar. The [Telegram Bot API server](https://github.com/tdlib/telegram-bot-api) does not
   provide information about file download progress (See https://github.com/tdlib/telegram-bot-api/issues/37).
   The workaround is to take statistics for the last N downloads and tries to calculate an approximation. 
