# minecraft-telegram-bot

A Forge mod for Minecraft 1.7.10 that syncs in-game chat with an
associated Telegram chat group.
The bot consists of two parts:

- A mod written in Java
- A bot program written in Rust

For the sake of simplicity, the two components simply communicates
through standard I/O by passing JSON messages back and forth.
