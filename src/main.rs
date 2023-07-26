use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use serenity::model::id::ChannelId;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, user::User},
    prelude::*,
};

mod plugins;
use plugins::*;

static ENVIRONMENT_VARIABLE_NAME: &str = "KOROVA_TOKEN";
static MIN_PLAYERS: usize = 2;
static CHECKUP_WAITTIME: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 2);

struct Handler;

struct ChannelGather {
    players: Vec<User>,
}

impl ChannelGather {
    fn new() -> Self {
        ChannelGather { players: vec![] }
    }

    fn add(&mut self, player: &User) {
        if self.players.iter().map(|p| p.id).all(|id| id != player.id) {
            self.players.push(player.clone());
        }
    }

    fn del(&mut self, player: &User) {
        self.players.retain(|p| p.id != player.id);
    }

    fn play(&mut self) -> Result<(String, Vec<User>), String> {
        if self.players.len() < MIN_PLAYERS {
            Err(String::from("We need at least 2 players."))
        } else {
            let lines = vec![
                String::from("Get ready for the game. Let me summon everyone:"),
                self.players
                    .iter()
                    .map(|p| p.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(" | "),
                String::from("Good luck & have fun!"),
            ];
            let players = self.players.clone();
            self.players = vec![];
            Ok((lines.join("\n"), players))
        }
    }

    fn status(&self) -> String {
        if self.players.is_empty() {
            String::from("Nobody wants to play right now. Write `!add` to join.")
        } else {
            let mut lines = vec![
                String::from("Ready players:"),
                self.players
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
                    .join(" | "),
            ];
            if self.players.len() >= MIN_PLAYERS {
                lines.push(String::from("Write `!play` to start the game."));
            }
            lines.join("\n")
        }
    }
}

struct GlobalGather;

impl TypeMapKey for GlobalGather {
    type Value = Arc<RwLock<HashMap<ChannelId, ChannelGather>>>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let plugins: Vec<Box<dyn Plugin + Send + Sync>> = vec![
            Box::new(PingPlugin),
            Box::new(ExcusePlugin),
            Box::new(WeatherPlugin {
                city: City {
                    name: "Warsaw",
                    name_locative: "w Warszawie",
                    country_code: "PL",
                },
            }),
            Box::new(WeatherPlugin {
                city: City {
                    name: "Dublin",
                    name_locative: "w Dublinie",
                    country_code: "IE",
                },
            }),
            Box::new(FortunePlugin {
                term: ",_,",
                triggers: vec![",_,"],
                error_msg: "Neeeciooor! Coś się popsuło (╯°□°）╯︵ ┻━┻",
            }),
            Box::new(FortunePlugin {
                term: "fortunka",
                triggers: vec!["!fortunka", "!f"],
                error_msg: "Nie ma fortunek, bo są błędy",
            }),
            Box::new(SadFortuneAdderPlugin {
                trigger: "!dodaj ,_, ",
            }),
            Box::new(CurrencyPlugin),
            Box::new(RandomSourceCodeLinePlugin),
        ];

        let type_map = ctx.data.read().await;
        let lock = type_map.get::<GlobalGather>().unwrap().clone();
        let mut map = lock.write().await;
        let gather = map.entry(msg.channel_id).or_insert_with(ChannelGather::new);

        let mut responses: Vec<String> = vec![];
        let response: Option<String> = match &msg.content[..] {
            "!add" => {
                gather.add(&msg.author);
                Some(gather.status())
            }
            "!del" => {
                gather.del(&msg.author);
                Some(gather.status())
            }
            "!play" => match gather.play() {
                Ok((text, players)) => {
                    schedule_checkup(&ctx, &msg.channel_id, &players);
                    Some(text)
                }
                Err(text) => Some(text),
            },
            "!status" => Some(gather.status()),
            "!help" => {
                let lines = vec![
                    "Gather commands: `!add`, `!del`, `!play`, `!status`.",
                    "Fortune commands: `,_,` (sad), `!fortunka` (classic).",
                    "Misc. commands: `!help`, `!code`, `!currency`, `!ping`, `!weather`, `!wymówka`.",
                ];
                Some(lines.join("\n"))
            }
            _ => None,
        };
        if let Some(r) = response {
            responses.push(r);
        }
        for p in plugins.iter() {
            if let Some(r) = p.handle(&msg).await {
                responses.push(r);
            }
        }

        for r in responses {
            if let Err(e) = msg.channel_id.say(&ctx.http, r).await {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        eprintln!("{} is connected!", ready.user.name);
    }
}

fn schedule_checkup(ctx: &Context, channel_id: &serenity::model::id::ChannelId, on: &Vec<User>) {
    let ctx = ctx.to_owned();
    let channel_id = channel_id.to_owned();
    let msg = format!(
        "Hey, hey! {}, it has been 2 hours since you started playing! \
        Remember to hydrate, take some rest, or possibly call it a day.",
        on.iter()
            .map(|p| p.mention().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    tokio::task::spawn(async move {
        tokio::time::sleep(CHECKUP_WAITTIME).await;
        if let Err(e) = channel_id.say(&ctx.http, msg).await {
            eprintln!("Error sending message: {:?}", e);
        };
    });
}

#[tokio::main]
async fn main() {
    let token = env::var(ENVIRONMENT_VARIABLE_NAME).unwrap_or_else(|_| {
        panic!(
            "Missing Discord bot token in {} environment variable.",
            ENVIRONMENT_VARIABLE_NAME
        )
    });

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<GlobalGather>(Arc::new(RwLock::new(HashMap::new())));
    }

    if let Err(e) = client.start().await {
        eprintln!("Client error: {:?}", e);
    }
}
