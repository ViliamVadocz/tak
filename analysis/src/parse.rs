use regex::Regex;
use tak::{takparse::Tps, *};

lazy_static! {
    // (count)(square)(direction)(drop counts)(stone)
    static ref TURN_MOVE_RE: Regex = Regex::new(r"([1-9]*)([a-z][1-9])([<>+-])([1-9]*)").unwrap();
    // (stone)(square)
    static ref TURN_PLACE_RE: Regex = Regex::new(r"([CS]?)([a-z][1-9])").unwrap();
    static ref OPTIONS_RE: Regex = Regex::new(r#"\[(\S+) ["'](.*?)["']\]"#).unwrap();
    static ref COMMENTS_RE: Regex = Regex::new(r"\{.*?\}").unwrap();
    static ref PLY_SPLIT_RE: Regex = Regex::new(r"\s*\d*\. |\s+|1-0|R-0|F-0|0-1|0-R|0-F|1/2-1/2|--").unwrap();
}

pub fn parse_ptn<const N: usize>(s: &str) -> Result<(Game<N>, Vec<Move>), Box<dyn std::error::Error>> {
    // parse game options
    let mut komi = 0;
    let (mut stones, mut caps) = default_starting_stones(N);
    let mut game = Game::default();
    let mut used_tps = false;
    for option in OPTIONS_RE.captures_iter(s) {
        let key = &option[1];
        let value = &option[2];
        match key {
            "Komi" => komi = value.parse::<i8>()?,
            "Flats" => stones = value.parse::<u8>()?,
            "Caps" => caps = value.parse::<u8>()?,
            "Size" => {
                if value.parse::<usize>()? != N {
                    Err(format!("game size mismatch, expected size {N} and found {value}"))?;
                }
            }
            "TPS" => {
                let tps: Tps = value.parse()?;
                game = tps.into();
                used_tps = true;
            }
            _ => {}
        }
    }
    game.half_komi = 2 * komi;
    if !used_tps {
        game.white_caps = caps;
        game.black_caps = caps;
        game.white_stones = stones;
        game.black_stones = stones;
    }

    // remove comments
    let s = OPTIONS_RE.replace_all(s, "");
    let s = COMMENTS_RE.replace_all(&s, "");

    // get individual plies (split at move numbers, space, and game result)
    let moves = PLY_SPLIT_RE
        .split(&s)
        .filter(|ss| !ss.is_empty())
        .map(|ss| ss.parse::<Move>())
        .collect::<Result<Vec<_>, _>>()?;

    Ok((game, moves))
}

pub fn parse_position<const N: usize>(s: &str) -> Result<Game<N>, Box<dyn std::error::Error>> {
    let mut iter = s.split(';');
    let mut game: Game<N> = iter.next().ok_or("missing tps")?.parse::<Tps>()?.into();
    if let Some(white_stones) = iter.next() {
        game.white_stones = white_stones.parse()?;
        game.white_caps = iter.next().ok_or("missing white caps")?.parse()?;
        game.black_stones = iter.next().ok_or("missing black stones")?.parse()?;
        game.black_caps = iter.next().ok_or("missing black caps")?.parse()?;
        game.half_komi = iter.next().ok_or("missing half komi")?.parse()?;
    } else {
        println!("Assuming standard reserve counts and Komi 2");
        game.half_komi = 4;
    }

    Ok(game)
}
