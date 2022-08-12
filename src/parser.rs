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
