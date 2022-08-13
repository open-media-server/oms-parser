use tmdb_client::{
    apis::client::APIClient,
    models::{SeasonDetails, TvDetails, TvObject},
};

use crate::BaseConfig;

pub fn set_metadata(config: &mut BaseConfig) {
    let client = APIClient::new_with_api_key("1506c013386779f969954794da85ac45");

    for show in &mut config.media {
        let tmdb_show = match get_show_metadata_from_dirty_name(&client, &show.name) {
            Some(metadata) => metadata,
            None => continue,
        };

        let tmdb_show_id = match tmdb_show.id {
            Some(id) => id,
            None => continue,
        };

        let tmdb_show_details = match get_show_details(&client, tmdb_show_id) {
            Some(details) => details,
            None => continue,
        };

        if let Some(name) = tmdb_show.name {
            show.name = name;
        }

        if let Some(id) = tmdb_show.id {
            show.id = id;
        }

        show.description = tmdb_show.overview;
        show.original_name = tmdb_show.original_name;
        show.air_date = tmdb_show.first_air_date;
        show.rating = tmdb_show.rating;

        let tmdb_seasons = match tmdb_show_details.seasons {
            Some(seasons) => seasons,
            None => continue,
        };

        for season in &mut show.seasons {
            let tmdb_season = match tmdb_seasons
                .iter()
                .find(|s| s.season_number.unwrap_or(0) == season.number)
            {
                Some(season) => season,
                None => continue,
            };

            let tmdb_show_id = match tmdb_show.id {
                Some(id) => id,
                None => continue,
            };

            let tmdb_season_number = match tmdb_season.season_number {
                Some(number) => number,
                None => continue,
            };

            let tmdb_season_details =
                match get_season_details(&client, tmdb_show_id, tmdb_season_number) {
                    Some(details) => details,
                    None => continue,
                };

            if let Some(name) = tmdb_season_details.name {
                season.name = name;
            }

            season.air_date = tmdb_season_details.air_date;

            let tmdb_episodes = match tmdb_season_details.episodes {
                Some(episodes) => episodes,
                None => continue,
            };

            for episode in &mut season.episodes {
                let tmdb_episode = match tmdb_episodes
                    .iter()
                    .find(|e| e.episode_number.unwrap_or(0) == episode.number)
                {
                    Some(episode) => episode,
                    None => continue,
                };

                if let Some(name) = &tmdb_episode.name {
                    episode.name = name.to_string();
                }
            }
        }
    }
}

fn get_show_metadata_from_dirty_name(client: &APIClient, name: &str) -> Option<TvObject> {
    match get_show_metadata(client, name) {
        Some(metadata) => return Some(metadata),
        None => (),
    }

    let mut parts = name.split_whitespace().collect::<Vec<&str>>();
    parts.pop();

    return get_show_metadata_from_dirty_name(client, &parts.join(" "));
}

fn get_show_metadata(client: &APIClient, name: &str) -> Option<TvObject> {
    let response = client
        .search_api()
        .get_search_tv_paginated(name, None, None, None)
        .ok()?;

    let shows = response.results?;

    if shows.is_empty() {
        return None;
    }

    return Some(shows[0].clone());
}

fn get_show_details(client: &APIClient, id: i32) -> Option<TvDetails> {
    client.tv_api().get_tv_details(id, None, None, None).ok()
}

fn get_season_details(client: &APIClient, id: i32, season_number: i32) -> Option<SeasonDetails> {
    client
        .tv_seasons_api()
        .get_tv_season_details(id, season_number, None, None, None)
        .ok()
}
