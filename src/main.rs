use tracing::{error, warn, info, trace};
use futures::stream::StreamExt;
use twilight_http::Client;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event, Intents};
use twilight_model::channel::{GuildChannel, Channel, TextChannel};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();
    
    let scheme = ShardScheme::Auto;

    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;

    let token = std::env::var("DISCORD_TOKEN").expect("piss off");

    let cluster = Cluster::builder(&token, intents)
        .shard_scheme(scheme)
        .build()
        .await.unwrap();

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = Client::new(&token);

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let mut events = cluster.events();
    
    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);
        tokio::spawn(handle_event(shard_id, event, http.clone()));
    }
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Client,
) {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            if msg.author.bot {
                return;
            }
            if let Some(Channel::Guild(GuildChannel::Text(channel))) = http.channel(msg.channel_id).await.unwrap() {
                http.create_message(msg.channel_id).content(format!("{}#{} said \"{}\" in {}", &msg.author.name, &msg.author.discriminator, &msg.content, &channel.name)).unwrap().await.unwrap();
            }
        }
        Event::ShardConnected(_) => {
            info!("Connected on shard {}", shard_id);
        }
        _ => {}
    }
}