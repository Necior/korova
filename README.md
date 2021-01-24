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

Now server admin can invite the bot to the server using the following URL:

```
https://discord.com/api/oauth2/authorize?client_id=CLIENT_ID&permissions=67584&scope=bot
```

where `CLIENT_ID` is your application's `client_id`.
The above URL can be also generated from the Discord application dashboard.
Visit OAuth2 tab, select `bot` scope and both `Send Messages` `Read Messages History` bot permissions to generate the URL.

## Update

Update repository (`git pull`), rebuild (`cargo build --release`) and restart the bot.

