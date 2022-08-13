use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, vec};

use dotenv::dotenv;
use metadata::set_metadata;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structure::{parse_structure, Node};
use tmdb_client::models::Credits;
use torrent_name_parser::Metadata;

use crate::parser::parse_season_number;
use crate::s3::parse_s3_structure;

pub mod metadata;
pub mod parser;
pub mod s3;
pub mod structure;

#[derive(Serialize, Deserialize)]
pub struct BaseConfig {
    base_url: String,
    media: Vec<Show>,
}

#[derive(Serialize, Deserialize)]
struct Show {
    name: String,
    seasons: Vec<Season>,
    id: i32,
    description: Option<String>,
    original_name: Option<String>,
    air_date: Option<String>,
    rating: Option<i32>,
    credits: Option<Credits>,
}

impl Show {
    fn create(name: &str) -> Show {
        let mut rng = rand::thread_rng();
        Show {
            name: name.to_string(),
            seasons: vec![],
            id: rng.gen_range(100000..1000000),
            description: None,
            original_name: None,
            air_date: None,
            rating: None,
            credits: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Season {
    name: String,
    number: i32,
    id: i32,
    episodes: Vec<Episode>,
    air_date: Option<String>,
}

impl Season {
    fn create(number: i32) -> Season {
        let mut rng = rand::thread_rng();
        Season {
            name: format!("Season {}", number),
            number: number,
            id: rng.gen_range(100000..1000000),
            episodes: vec![],
            air_date: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Episode {
    name: String,
    number: i32,
    id: i32,
    path: String,
}

impl Episode {
    fn create(name: &str, number: i32, path: &str) -> Episode {
        let mut rng = rand::thread_rng();
        Episode {
            number: number,
            id: rng.gen_range(100000..1000000),
            name: name.to_string(),
            path: path.to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let node = &parse_s3_structure().await.children.unwrap()[0];

    // let root_path = PathBuf::from_str("media").expect("Failed to parse root path");

    // let node = parse_structure(&root_path);

    let mut config = parse_show_tree(&node);

    set_metadata(&mut config);

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        fs::write("./config.json", json).unwrap();
    }
}

fn parse_show_tree(node: &Node) -> BaseConfig {
    let mut config = BaseConfig {
        base_url: "https://s3.eu-central-1.wasabisys.com/aerio-media".to_string(),
        media: vec![],
    };

    for show_node in node.children.as_ref().unwrap() {
        let metadata = Metadata::from(&show_node.name).unwrap();
        let name = metadata.title();

        let show_index = match config.media.iter().position(|m| m.name == name) {
            Some(show) => show,
            None => {
                let show = Show::create(name);
                config.media.push(show);
                config.media.len() - 1
            }
        };

        let show = &mut config.media[show_index];

        for season_node in show_node.children.as_ref().unwrap() {
            let season_number = parse_season_number(&season_node.name).unwrap_or(1);

            let season_index = match show.seasons.iter().position(|s| s.number == season_number) {
                Some(season) => season,
                None => {
                    let season = Season::create(season_number);
                    show.seasons.push(season);
                    show.seasons.len() - 1
                }
            };

            let season = &mut show.seasons[season_index];

            // For shows without a season folder
            if season_node.children.is_none() {
                season.episodes.push(parse_episode(&season_node));
                continue;
            }

            for episode_node in season_node.children.as_ref().unwrap() {
                season.episodes.push(parse_episode(&episode_node));
            }
        }
    }

    return config;
}

fn parse_episode(node: &Node) -> Episode {
    if let Ok(metadata) = Metadata::from(&node.name) {
        let episode_number = metadata.episode().unwrap_or(0);

        return Episode::create(metadata.title(), episode_number, &node.path);
    }

    Episode::create("", 0, &node.path)
}
