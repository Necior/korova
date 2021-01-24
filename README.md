# korova

`korova` is a Discord bot (written in Rust). It's main functionality is gathering people to play games.

## Build

```bash
git clone https://github.com/Necior/korova
cd korova
cargo build --release
```

Now `./target/release/korova` contains the binary

## Usage

You have to [create a Discord app](https://discord.com/developers/applications), create app's bot and provide its token as a `KOROVA_TOKEN` environment variable:

```bash
KOROVA_TOKEN="put your token here" ./target/release/korova
```

## Update

Update repository (`git pull`), rebuild (`cargo build --release`) and restard the bot.

