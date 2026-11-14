use poise::serenity_prelude as serenity;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

// 1. État partagé de l'application (remplace totalement TypeMapKey)
struct Data {
    db: SqlitePool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Commande hybride : fonctionne en slash command (/ping) ET en commande texte (!ping)
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // Accès direct et sans verrou au pool SQLite
    let pool = &ctx.data().db;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bot_config")
        .fetch_one(pool)
        .await?;

    ctx.say(format!("Pong ! ({count} entrée(s) en base)")).await?;
    Ok(())
}

/// Récupère le token Discord depuis SQLite
async fn fetch_discord_token(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT value FROM bot_config WHERE key = ?")
        .bind("discord_token")
        .fetch_one(pool)
        .await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 2. Connexion moderne à SQLite : crée le fichier bot_data.db automatiquement s'il n'existe pas
    let connect_options = SqliteConnectOptions::from_str("sqlite:bot_data.db")?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Initialisation de la table de configuration
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bot_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
        .execute(&pool)
        .await?;

    // 3. Extraction du token
    let token = fetch_discord_token(&pool)
        .await
        .expect("Token introuvable dans la table bot_config (clé: 'discord_token')");

    // 4. Intents recommandés par la doc officielle Poise 0.7
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    // 5. Construction du framework Poise
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("{} est connecté et prêt !", ready.user.name);
                // Enregistre automatiquement toutes les commandes slash auprès de Discord
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db: pool })
            })
        })
        .build();

    // 6. Démarrage du client Discord
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    Ok(())
}