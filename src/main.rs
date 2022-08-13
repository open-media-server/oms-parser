use std::{
    fs::{self, DirEntry},
    path::PathBuf,
    str::FromStr,
    vec,
};

use metadata::set_metadata;
use serde::{Deserialize, Serialize};
use tmdb_client::models::Credits;
use torrent_name_parser::Metadata;

use crate::parser::parse_season_number;

pub mod metadata;
pub mod parser;

#[derive(Serialize, Deserialize)]
pub struct BaseConfig {
    media: Vec<Show>,
}

#[derive(Serialize, Deserialize)]
struct Show {
    name: String,
    seasons: Vec<Season>,
    description: Option<String>,
    original_name: Option<String>,
    air_date: Option<String>,
    rating: Option<i32>,
    credits: Option<Credits>,
}

impl Show {
    fn create(name: &str) -> Show {
        Show {
            name: name.to_string(),
            seasons: vec![],
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
    episodes: Vec<Episode>,
    air_date: Option<String>,
}

impl Season {
    fn create(number: i32) -> Season {
        Season {
            name: format!("Season {}", number),
            number: number,
            episodes: vec![],
            air_date: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Episode {
    name: String,
    number: i32,
    path: String,
}

impl Episode {
    fn create(name: &str, number: i32, path: &str) -> Episode {
        Episode {
            number: number,
            name: name.to_string(),
            path: path.to_string(),
        }
    }
}

fn main() {
    let root_path = PathBuf::from_str("./media").expect("Failed to parse root path");
    let mut config = parse_show_tree(&root_path);

    set_metadata(&mut config);

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        // println!("{}", json);
        fs::write("./out.json", json).unwrap();
    }
}

fn parse_show_tree(path: &PathBuf) -> BaseConfig {
    let mut config = BaseConfig { media: vec![] };

    let show_paths = path
        .read_dir()
        .expect("Failed to read the root directory")
        .filter_map(|p| p.ok());

    for show_path in show_paths {
        let file_name_result = show_path.file_name();
        let file_name = match file_name_result.to_str() {
            Some(n) => n,
            None => continue,
        };

        let metadata = Metadata::from(file_name).unwrap();
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

        let season_paths = show_path.path().read_dir().unwrap().filter_map(|p| p.ok());

        for season_path in season_paths {
            let season_number =
                parse_season_number(season_path.file_name().to_str().unwrap()).unwrap_or(1);

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
            if season_path.path().is_file() {
                season.episodes.push(parse_episode(season_path));
                continue;
            }

            let episode_paths = season_path
                .path()
                .read_dir()
                .unwrap()
                .filter_map(|p| p.ok());

            for episode_path in episode_paths {
                season.episodes.push(parse_episode(episode_path));
            }
        }
    }

    return config;
}

fn parse_episode(path: DirEntry) -> Episode {
    let metadata = Metadata::from(path.file_name().to_str().unwrap()).unwrap();

    let episode_number = metadata.episode().unwrap_or(0);

    Episode::create(
        metadata.title(),
        episode_number,
        path.path().to_str().unwrap(),
    )
}
