use regex::Regex;

pub fn parse_season_number(name: &str) -> Option<i32> {
    if name.to_lowercase().contains("specials") {
        return Some(0);
    }

    let parts = name.split(" ").collect::<Vec<&str>>();

    // Match names with "season ##" in them
    match parts.iter().position(|p| p.to_lowercase() == "season") {
        Some(index) => match parts[index + 1].parse::<i32>() {
            Ok(n) => return Some(n),
            Err(_) => (),
        },
        None => (),
    };

    // Match names with "s##" in them
    match parts.iter().find(|p| p.to_lowercase().starts_with("s")) {
        Some(p) => match p
            .chars()
            .filter(|c| c.is_digit(10))
            .collect::<String>()
            .parse::<i32>()
        {
            Ok(n) => return Some(n),
            Err(_) => (),
        },
        None => (),
    };

    None
}

pub fn parse_episode_number(name: &str) -> Option<i32> {
    let regex = Regex::new(
        r"(?i)(?:e|episode)[^.\d]?(?P<short>\d{1,3})|\d+x(?P<cross>\d+)|s\d+ - (?P<dash>\d+)",
    )
    .unwrap();

    let captures = regex.captures(name).unwrap();

    captures
        .name("short")
        .or_else(|| captures.name("cross"))
        .or_else(|| captures.name("dash"))
        .map(|m| m.as_str().parse().unwrap_or(0))
}
