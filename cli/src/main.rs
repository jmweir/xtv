use clap::{
    Parser,
    Subcommand
};

use itertools::Itertools;

use client_lib::{
    Device,
    KeyCode,
    TuningTarget,
    XTVClient,
};


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    Channels {},
    Devices {},
    Exit {},
    FF {},
    Pause {},
    Play {},
    Recordings {},
    Rew {},
    Search {
        #[clap(value_parser)]
        query: String
    },
    Stop {},
    Token {},
    Tune {
        #[clap(value_enum)]
        target: TuningTarget,

        #[clap(value_parser)]
        id: String
    },
}

async fn channels(client: &XTVClient) -> Result<(), Box<dyn std::error::Error>> {
    let channel_map = client.channels().await?;
    for call_sign in channel_map.keys().sorted() {
        let channels = &channel_map[call_sign];
        println!("{} ({}): {:?}", call_sign, channels[0].name(), channels.iter().map(|c| c.number()).collect::<Vec<u16>>());
    }
    Ok(())
}

async fn devices(client: &XTVClient) -> Result<(), Box<dyn std::error::Error>> {
    let device_map = client.devices().await?;
    device_map.values().for_each(|v| println!("{} {}", v.id(), v.name()));
    Ok(())
}

async fn recordings(client: &XTVClient, device: &Device) -> Result<(), Box<dyn std::error::Error>> {
    let recordings = client.recordings(device).await?;
    recordings.iter().for_each(|rec| println!("{} {} {}", rec.title(), rec.date_recorded(), rec.media_id()));
    Ok(())
}

async fn search(client: &XTVClient, query: &String) -> Result<(), Box<dyn std::error::Error>> {
    let search_results = client.search(query).await?;
    search_results.iter().for_each(|res| println!("{}: {}", res.name(), res.subtitle()));
    Ok(())
}

async fn token(client: &XTVClient) -> Result<(), Box<dyn std::error::Error>> {
    let token = client.token().await?;
    println!("{}", token);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cli = Cli::parse();
    let mut client = XTVClient::new()?;
    let device = client.lookup_device("Media Room").await?;

    match &cli.command {
        Some(Commands::Channels {}) => { channels(&client).await?; }
        Some(Commands::Devices {}) => { devices(&client).await?; }
        Some(Commands::Exit {}) => { client.press_key(KeyCode::Exit, &device).await?; }
        Some(Commands::FF {}) => { client.press_key(KeyCode::FastForward, &device).await?; }
        Some(Commands::Pause {}) => { client.press_key(KeyCode::Pause, &device).await?; }
        Some(Commands::Play {}) => { client.press_key(KeyCode::Play, &device).await?; }
        Some(Commands::Recordings {}) => { recordings(&client, &device).await?; }
        Some(Commands::Rew {}) => { client.press_key(KeyCode::Rewind, &device).await?; }
        Some(Commands::Search { query }) => { search(&client, query).await?; }
        Some(Commands::Stop {}) => { client.press_key(KeyCode::Stop, &device).await?; }
        Some(Commands::Token {}) => { token(&mut client).await?; }
        Some(Commands::Tune { target, id }) => { client.tune(target, id, &device).await?; }
        None => ()
    };

    Ok(())
}
