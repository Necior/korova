use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use serenity::futures::TryStreamExt;
use serenity::model::id::ChannelId;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, user::User},
    prelude::*,
};
use tokio::time::Duration;

static ENVIRONMENT_VARIABLE_NAME: &str = "KOROVA_TOKEN";
static MIN_PLAYERS: usize = 2;

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

    fn play(&mut self) -> String {
        if self.players.len() < MIN_PLAYERS {
            String::from("We need at least 2 players.")
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
            self.players = vec![];
            lines.join("\n")
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

fn get_weather() -> String {
    if let Ok(apikey) = env::var("KOROVA_OWM_APIKEY") {
        match &openweathermap::blocking::weather("Warsaw,PL", "metric", "pl", &apikey) {
            Ok(current) => {
                let desc = current.weather[0].description.to_string();
                let temp = format!("{}°C", current.main.temp);
                let pres = format!("{} hPa", current.main.pressure);
                format!("Pogoda w Warszawie: {}, {}, {}.", desc, temp, pres)
            }
            Err(e) => format!(
                "Coś się, coś się popsuło i nie było mnie słychać… (Informacja dla nerdów: {}.)",
                e
            ),
        }
    } else {
        "*chlip* *chlip*, jak mam sprawdzić pogodę, jeśli nie mam klucza do API?".to_string()
    }
}

async fn get_mongodb_collection() -> Option<mongodb::Collection<mongodb::bson::Document>> {
    let connection_string = env::var("KOROVA_MONGODB_CONNECTION_STRING").ok()?;
    let db_name = env::var("KOROVA_MONGODB_DB").ok()?;
    let collection_name = env::var("KOROVA_MONGODB_COLLECTION").ok()?;
    let client_options = mongodb::options::ClientOptions::parse(&connection_string)
        .await
        .ok()?;
    let client = mongodb::Client::with_options(client_options).unwrap();
    let db = client.database(&db_name);
    Some(db.collection::<mongodb::bson::Document>(&collection_name))
}

async fn get_fortune(term: &str) -> Option<String> {
    let collection = get_mongodb_collection().await?;
    let mut cursor = collection
        .aggregate(
            vec![
                mongodb::bson::doc! {"$match": {"term": term}},
                mongodb::bson::doc! {"$sample": {"size": 1}},
            ],
            None,
        )
        .await
        .ok()?;
    let el = cursor.try_next().await.ok()??;
    Some(el.get("description")?.as_str()?.to_string())
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let type_map = ctx.data.read().await;
        let lock = type_map.get::<GlobalGather>().unwrap().clone();
        let mut map = lock.write().await;
        let gather = map.entry(msg.channel_id).or_insert_with(ChannelGather::new);

        let response: Option<String> = match &msg.content[..] {
            "!add" => {
                gather.add(&msg.author);
                Some(gather.status())
            }
            "!del" => {
                gather.del(&msg.author);
                Some(gather.status())
            }
            "!play" => Some(gather.play()),
            "!status" => Some(gather.status()),
            "!help" => {
                let lines = vec![
                    "Gather commands: `!add`, `!del`, `!play`, `!status`.",
                    "Fortune commands: `,_,` (sad), `!fortunka` (classic).",
                    "Misc. commands: `!help`, `!ping`, `!weather`, `!wymówka`.",
                ];
                Some(lines.join("\n"))
            }
            "!ping" => Some(format!("Pong, {}.", msg.author.mention())),
            "xD" => {
                let c = ctx.clone();
                let m = msg.clone();
                tokio::task::spawn(async move {
                    enum Action<'a> {
                        Sleep(Duration),
                        SendMsg(&'a str),
                    }

                    let actions = [
                        Action::Sleep(Duration::from_secs(20)),
                        Action::SendMsg("xDDD"),
                        Action::Sleep(Duration::from_secs(10)),
                        Action::SendMsg("No nie mogę, skisłem z tego straszie"),
                        Action::Sleep(Duration::from_secs(60 * 60 * 40)),
                        Action::SendMsg("Nudzi mi się. Może sobie poczytam coś (oby) zabawnego?"),
                        Action::SendMsg("!fortunka"),
                    ];
                    for action in &actions {
                        match action {
                            Action::Sleep(duration) => {
                                tokio::time::sleep(*duration).await;
                            }
                            Action::SendMsg(msg) => {
                                if let Err(e) = m.channel_id.say(&c.http, msg).await {
                                    eprintln!("Error sending message: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                });
                None
            }
            "!w" | "!wymówka" => match get_fortune("wymówka").await {
                Some(s) => Some(s),
                None => Some(
                    "Dziwne, nie znalazłem żadnej wymówki. Pewnie Necior coś popsuł.".to_string(),
                ),
            },
            "!pogoda" | "!weather" => Some(get_weather()),
            ",_," => match get_fortune(",_,").await {
                Some(s) => Some(s),
                None => Some("Neeeciooor! Coś się popsuło (╯°□°）╯︵ ┻━┻".to_string()),
            },
            "!f" | "!fortunka" => match get_fortune("fortunka").await {
                Some(s) => Some(s),
                None => Some("Nie ma fortunek, bo są błędy.".to_string()),
            },
            _ => None,
        };

        if let Some(r) = response {
            if let Err(e) = msg.channel_id.say(&ctx.http, r).await {
                eprintln!("Error sending message: {:?}", e);
            }
        };
    }

    async fn ready(&self, _: Context, ready: Ready) {
        eprintln!("{} is connected!", ready.user.name);
    }
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
