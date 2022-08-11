mod parser;

use std::{path::PathBuf, str::FromStr, vec};

use parser::parse_season;
use serde::{Deserialize, Serialize};
use torrent_name_parser::Metadata;

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
    path: String,
}

fn main() {
    // build_config();

    let season =
        parse_season("Kaguya-sama Love is War S02 1080p Dual Audio BDRip 10 bits AAC x265-EMBER")
            .unwrap_or(0);

    println!("{}", season);
}

fn parse_file_tree(path: &PathBuf) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = vec![];

    let curr_paths = path.read_dir().unwrap().map(|m| m.unwrap().path());

    for path in curr_paths {
        if path.is_dir() {
            paths.append(&mut parse_file_tree(&path));
        } else {
            paths.push(path);
        }
    }

    paths
}

fn build_config() {
    let mut config = BaseConfig { media: vec![] };

    let path = PathBuf::from_str("./media").unwrap();

    let paths = parse_file_tree(&path);

    for path in paths {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let metadata = Metadata::from(file_name).unwrap();

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

        let season_number = metadata.season().unwrap_or(0);

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

        let episode_number = metadata.episode().unwrap_or(0);

        let _episode_index = match season
            .episodes
            .iter()
            .position(|e| e.number == episode_number)
        {
            Some(episode) => episode,
            None => {
                let episode = Episode {
                    number: episode_number,
                    path: String::from_str(path.to_str().unwrap()).unwrap(),
                };
                season.episodes.push(episode);
                season.episodes.len() - 1
            }
        };
    }

    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", json);
}
