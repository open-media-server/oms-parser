use std::{
    fs::{self, DirEntry},
    io::Error,
    path::PathBuf,
    str::FromStr,
    vec,
};

use serde::{Deserialize, Serialize};
use torrent_name_parser::Metadata;

use crate::parser::parse_season_number;

pub mod parser;

#[derive(Serialize, Deserialize)]
struct BaseConfig {
    media: Vec<Show>,
}

#[derive(Serialize, Deserialize)]
struct Show {
    title: String,
    seasons: Vec<Season>,
}

#[derive(Serialize, Deserialize)]
struct Season {
    number: i32,
    episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize)]
struct Episode {
    number: i32,
    title: String,
    path: String,
}

fn main() {
    let root_path = PathBuf::from_str("./media").unwrap();
    let config = parse_show_tree(&root_path).unwrap();

    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", json);

    fs::write("./out.json", json).unwrap();
}

fn parse_show_tree(path: &PathBuf) -> Result<BaseConfig, Error> {
    let mut config = BaseConfig { media: vec![] };

    let show_paths = path.read_dir()?.filter_map(|p| p.ok());

    for show_path in show_paths {
        let metadata = Metadata::from(show_path.file_name().to_str().unwrap()).unwrap();
        let title = metadata.title();

        let show_index = match config.media.iter().position(|m| m.title == title) {
            Some(show) => show,
            None => {
                let show = Show {
                    title: title.to_string(),
                    seasons: vec![],
                };
                config.media.push(show);
                config.media.len() - 1
            }
        };

        let show = &mut config.media[show_index];

        let season_paths = show_path.path().read_dir()?.filter_map(|p| p.ok());

        for season_path in season_paths {
            let season_number =
                parse_season_number(season_path.file_name().to_str().unwrap()).unwrap_or(1);

            let season_index = match show.seasons.iter().position(|s| s.number == season_number) {
                Some(season) => season,
                None => {
                    let season = Season {
                        number: season_number,
                        episodes: vec![],
                    };
                    show.seasons.push(season);
                    show.seasons.len() - 1
                }
            };

            let season = &mut show.seasons[season_index];

            if season_path.path().is_file() {
                season.episodes.push(parse_episode(season_path));
                continue;
            }

            let episode_paths = season_path.path().read_dir()?.filter_map(|p| p.ok());

            for episode_path in episode_paths {
                season.episodes.push(parse_episode(episode_path));
            }
        }
    }

    return Ok(config);
}

fn parse_episode(path: DirEntry) -> Episode {
    let metadata = Metadata::from(path.file_name().to_str().unwrap()).unwrap();

    let episode_number = metadata.episode().unwrap();

    Episode {
        number: episode_number,
        title: metadata.title().to_string(),
        path: String::from_str(path.path().to_str().unwrap()).unwrap(),
    }
}
