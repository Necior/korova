use std::collections::HashMap;
use std::env;
use std::result::Result;

use serenity::futures::TryStreamExt;
use serenity::{async_trait, model::channel::Message, prelude::*};

// Plugin is used for simple plugins which (optionally) respond with a single message when
// triggered with an incoming message.
#[async_trait]
pub trait Plugin {
    async fn handle(self: &Self, incoming_message: &Message) -> Option<String>;
}

// A plugin which prints a ~random line of code.
pub struct RandomSourceCodeLinePlugin;

#[async_trait]
impl Plugin for RandomSourceCodeLinePlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        if msg.content == "!code" {
            use rand::seq::SliceRandom;
            let source = include_str!("plugins.rs");
            let lines: Vec<_> = source.lines().filter(|line| line.trim().len() > 15).collect();
            Some(format!("```\n{}```", lines.choose(&mut rand::thread_rng()).unwrap().to_string()))
        } else {
            None
        }
    }
}

pub struct PingPlugin;

#[async_trait]
impl Plugin for PingPlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        if msg.content == "!ping" {
            Some(format!("Pong, {}.", msg.author.mention()))
        } else {
            None
        }
    }
}

pub struct ExcusePlugin;

#[async_trait]
impl Plugin for ExcusePlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        if msg.content == "!w" || msg.content == "!wymówka" {
            match get_fortune("wymówka").await {
                Some(s) => Some(s),
                None => Some(
                    "Dziwne, nie znalazłem żadnej wymówki. Pewnie Necior coś popsuł.".to_string(),
                ),
            }
        } else {
            None
        }
    }
}

pub struct City {
    pub name: &'static str,
    pub name_locative: &'static str,
    pub country_code: &'static str,
}

pub struct WeatherPlugin {
    pub city: City,
}

#[async_trait]
impl Plugin for WeatherPlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        if msg.content == "!pogoda" || msg.content == "!weather" {
            Some(get_weather(&self.city))
        } else {
            None
        }
    }
}

pub struct FortunePlugin {
    pub term: &'static str,
    pub triggers: Vec<&'static str>,
    pub error_msg: &'static str,
}

#[async_trait]
impl Plugin for FortunePlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        if self.triggers.contains(&&msg.content[..]) {
            match get_fortune(self.term).await {
                Some(s) => Some(s),
                None => Some(self.error_msg.to_string()),
            }
        } else {
            None
        }
    }
}

pub struct SadFortuneAdderPlugin {
    pub trigger: &'static str,
}

#[async_trait]
impl Plugin for SadFortuneAdderPlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        if msg.content.starts_with(self.trigger) {
            let f = &msg.content[self.trigger.len()..];
            if f.is_empty() {
                Some("Pustej nie dodaję.".to_string())
            } else {
                match add_fortune(",_,", f).await {
                    Some(()) => Some("Dodane :)".to_string()),
                    None => {
                        Some("Coś spadło z rowerka, szukaj kaczki do debuggowania.".to_string())
                    }
                }
            }
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize)]
struct RatesResponse {
    rates: HashMap<String, f64>,
}

pub struct CurrencyPlugin;

#[async_trait]
impl Plugin for CurrencyPlugin {
    async fn handle(self: &Self, msg: &Message) -> Option<String> {
        async fn get() -> Result<RatesResponse, reqwest::Error> {
            reqwest::get("https://open.er-api.com/v6/latest/PLN")
                .await?
                .json()
                .await
        }
        if msg.content == "!currency" {
            let rr = get().await;
            match rr {
                Err(e) => Some(format!("Error: {}", e)),
                Ok(rr) => Some(format!(
                    "**Currency exchange rates**\nEURPLN ≈ {:.3}\nUSDPLN ≈ {:.3}",
                    1.0 / rr.rates.get("EUR")?,
                    1.0 / rr.rates.get("USD")?
                )),
            }
        } else {
            None
        }
    }
}

fn get_weather(city: &City) -> String {
    if let Ok(apikey) = env::var("KOROVA_OWM_APIKEY") {
        match &openweathermap::blocking::weather(
            &format!("{},{}", city.name, city.country_code),
            "metric",
            "pl",
            &apikey,
        ) {
            Ok(current) => {
                let desc = current.weather[0].description.to_string();
                let temp = format!("{}°C", current.main.temp);
                let pres = format!("{} hPa", current.main.pressure);
                format!(
                    "Pogoda {}: {}, {}, {}.",
                    city.name_locative, desc, temp, pres
                )
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

async fn add_fortune(term: &str, description: &str) -> Option<()> {
    let collection = get_mongodb_collection().await?;
    collection
        .insert_one(
            mongodb::bson::doc! {"term": term, "description": description},
            None,
        )
        .await
        .ok()?;
    Some(())
}
