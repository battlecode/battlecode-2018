use super::*;
use super::super::world::Rounds;
use failure::Error;
use fnv::FnvHashMap;

/// Tokenizer.
struct Tok<'a> {
    line: usize,
    col: usize,
    cur: &'a str
}
impl<'a> Tok<'a> {
    #[inline(never)]
    fn chew(&mut self) -> Option<&'a str> {
        let mut eating = false;
        let mut eating_idx = 0usize;
        let mut comment = false;
        for (i, c) in self.cur.char_indices() {
            self.col += 1;
            let whitespace = c.is_whitespace();
            if eating {
                if c == '\n' {
                    // newline while eating, don't consume it
                    let item = &self.cur[eating_idx..i];
                    self.cur = &self.cur[i..];
                    self.col = 0;
                    return Some(item);
                }
                if whitespace {
                    let item = &self.cur[eating_idx..i];
                    self.cur = &self.cur[i+1..];
                    return Some(item);
                }
                if c == ':' || c == '=' {
                    let item = &self.cur[eating_idx..i];
                    self.cur = &self.cur[i..];
                    return Some(item);
                }
            } else {
                if c == '\n' {
                    // newline while not eating
                    self.cur = &self.cur[i+1..];
                    self.col = 0;
                    self.line += 1;
                    return None;
                }
                if c == '#' {
                    comment = true;
                }

                if !comment {
                    if c == '>' {
                        // switch modes
                        self.cur = &self.cur[i+1..];
                        return Some(">");
                    } else if !whitespace {
                        eating = true;
                        eating_idx = i;
                    }
                }
            }
        }
        // nothing interesting in the whole string
        self.cur = "";
        None
    }

    // Tokenizer used in maps.
    #[inline(never)]
    fn chew_map(&mut self) -> Option<char> {
        let mut comment = false;
        for (i, c) in self.cur.char_indices() {
            if c == '\n' {
                self.col = 0;
                self.line += 1;
                self.cur = &self.cur[i+1..];
                return None;
            }
            if c == '#' {
                comment = true;
            }
            if !comment && !c.is_whitespace() {
                self.cur = &self.cur[i + c.len_utf8()..];
                return Some(c);
            }
        }
        self.cur = "";
        None
    }
    fn has(&self) -> bool {
        self.cur != ""
    }
}

#[derive(Clone, Copy)]
enum Symmetry {
    Horiz,
    Vert,
    Spiral,
    None
}
fn get_opposite(sym: &Symmetry, loc: (usize, usize), width: usize, height: usize) -> Option<(usize, usize)> {
    match sym {
        &Symmetry::Horiz => Some((width - 1 - loc.0, loc.1)),
        &Symmetry::Vert => Some((loc.0, height - 1 - loc.1)),
        &Symmetry::Spiral => Some((width - 1 - loc.0, height - 1 - loc.1)),
        &Symmetry::None => None
    }
}

#[derive(Clone)]
struct Thing {
    passable: bool,
    team: Option<Team>,
    karbonite: u32
}

struct PlanetData {
    width: Option<usize>,
    height: Option<usize>,
    symmetry: Option<Symmetry>,
    cur_y: Option<usize>,
    things: FnvHashMap<(usize, usize), Thing>
}

pub(crate) fn parse_text_map(cur: &str) -> Result<GameMap, Error> {
    let mut tok = Tok {
        cur,
        line: 0,
        col: 0
    };

    let mut orbit_amplitude: Option<Rounds> = None;
    let mut orbit_period: Option<Rounds> = None;
    let mut orbit_center: Option<Rounds> = None;
    let mut seed: Option<u16> = None;

    let mut names = FnvHashMap::<char, Thing>::default();

    let mut cur_planet: Option<Planet> = None;
    let mut planets = [
        PlanetData { width: None, height: None, symmetry: None, cur_y: None, things: FnvHashMap::default() },
        PlanetData { width: None, height: None, symmetry: None, cur_y: None, things: FnvHashMap::default() }
    ];
    let mut asteroids: Vec<(usize, usize, usize, usize)> = vec![];

    while tok.has() {
        let start = tok.chew();
        if start.is_none() {
            continue;
        }
        let start = start.unwrap();
        if start == "EARTH" || start == "MARS" {
            if tok.chew() != Some(":") {
                bail!("{} must be followed by a colon!", start);
            }
            cur_planet = if start == "EARTH" {
                Some(Planet::Earth)
            } else {
                Some(Planet::Mars)
            };
            continue;
        }
        if start == ">" {
            if cur_planet.is_none() {
                bail!("need to set planet to draw map at line {} col {}", tok.line, tok.col);
            }
            let p = cur_planet.unwrap() as usize;

            let y = planets[p].cur_y;
            if y.is_none() {
                bail!("need to set height before drawing map at line {} col {}", tok.line, tok.col);
            }
            let y = y.unwrap();
            if y == 9999 {
                bail!("too many lines in map at line {} col {}", tok.line, tok.col);
            }
            let sym = planets[p].symmetry;
            if sym.is_none() {
                bail!("need to set symmetry before drawing map at line {} col {}", tok.line, tok.col);
            }
            let sym = sym.unwrap();
            let height = planets[p].height;
            if height.is_none() {
                bail!("need to set height before drawing map at line {} col {}", tok.line, tok.col);
            }
            let height = height.unwrap();
            let width = planets[p].width;
            if width.is_none() {
                bail!("need to set width before drawing map at line {} col {}", tok.line, tok.col);
            }
            let width = width.unwrap();

            let mut x = 0;
            while let Some(thing_) = tok.chew_map() {
                if x >= width {
                    bail!("map too wide at line {} col {}", tok.line, tok.col);
                }
                let thing = names.get(&thing_);
                if let Some(thing) = thing {
                    planets[p].things.insert((x, y), thing.clone());
                    if let Some(opposite) = get_opposite(&sym, (x, y), width, height) {
                        let mut otherthing = thing.clone();
                        // swap teams on inversion
                        if let Some(team) = thing.team {
                            otherthing.team = Some(team.other());
                        }
                        planets[p].things.insert(opposite, otherthing);
                    }
                } else {
                    bail!("unknown map symbol: {} at line {} col {}", thing_, tok.line, tok.col)
                }
                x += 1;
            }
            if y > 0 {
                planets[p].cur_y = Some(y-1);
            } else {
                planets[p].cur_y = Some(9999);
            }
            continue;
        }
        if start == "*" {
            let round = tok.chew();
            let x = tok.chew();
            let y = tok.chew();
            let karbonite = tok.chew();
            if let (Some(round), Some(x), Some(y), Some(karbonite)) = (round, x, y, karbonite) {
                if let (Ok(round), Ok(x), Ok(y), Ok(karbonite)) = (round.parse(), x.parse(), y.parse(), karbonite.parse()) {
                    asteroids.push((round, x, y, karbonite));
                } else {
                    bail!("failed to parse asteroid at line {}", tok.line)
                }
            } else {
                bail!("failed to parse asteroid at line {}", tok.line)
            }
            continue;
        }

        let sep = tok.chew();
        if sep.is_none() {
            bail!("expected = or : at line {} col {}", tok.line, tok.col);
        }
        let sep = sep.unwrap();
        if sep == "=" {
            assert_eq!(start.chars().count(), 1, "map symbols can only be one code point wide, {} is too long at line {} col {}",
                start, tok.line, tok.col);
            let mut thing = Thing {
                passable: true,
                team: None,
                karbonite: 0
            };
            while let Some(attr) = tok.chew() {
                match attr {
                    "red_worker" => thing.team = Some(Team::Red),
                    "blue_worker" => thing.team = Some(Team::Blue),
                    "impassable" => thing.passable = false,
                    a => if a.ends_with("k") {
                        let karb = a[..a.len() - 1].parse::<u32>();
                        if let Ok(karb) = karb {
                            thing.karbonite = karb;
                        } else {
                            bail!("failed to convert karbonite string \"{}\" to integer at line {}, col {}", a, tok.line, tok.col);
                        }
                    } else {
                        bail!("unknown attribute: {} (did you leave a 'k' off your integer?) at line {} col {}", a, tok.line, tok.col);
                    }
                }
            }
            names.insert(start.chars().nth(0).unwrap(), thing);
        } else if sep == ":" {
            let value = {
                if let Some(value) = tok.chew() {
                    value
                } else {
                    bail!("expected value for property {} at line {} col {}", start, tok.line, tok.col);
                }
            };
            if start == "symmetry" {
                if cur_planet.is_none() {
                    bail!("need to set planet to set symmetry at line {} col {}", tok.line, tok.col);
                }
                let p = cur_planet.unwrap() as usize;

                planets[p].symmetry = Some(match value {
                    "vert" | "vertical" | "v" => Symmetry::Vert,
                    "hor" | "horizontal" | "h" => Symmetry::Horiz,
                    "spiral" => Symmetry::Spiral,
                    "none" => Symmetry::None,
                    _ => bail!("unknown symmetry {} at line {} col {}", value, tok.line, tok.col)
                });
                continue;
            }
            let value = if let Ok(value) = value.parse::<usize>() {
                value
            } else {
                bail!("failed to parse {} as int at line {}, col {}", value, tok.line, tok.col);
            };

            if start == "width" || start == "height" {
                if cur_planet.is_none() {
                    bail!("need to set planet to set {} at line {} col {}", start, tok.line, tok.col);
                }
                let p = cur_planet.unwrap() as usize;
                match start {
                    "width" => planets[p].width = Some(value),
                    "height" => {
                        planets[p].cur_y = Some(value-1);
                        planets[p].height = Some(value);
                    },
                    _ => unreachable!()
                }
                continue;
            }
            match start {
                "orbit_amplitude" => orbit_amplitude = Some(value as Rounds),
                "orbit_period" => orbit_period = Some(value as Rounds),
                "orbit_center" => orbit_center = Some(value as Rounds),
                "seed" => seed = Some(value as u16),
                _ => bail!("unknown map property: {} at line {} col {}", start, tok.line, tok.col)
            }
        } 
    }

    let orbit_amplitude = if let Some(orbit_amplitude) = orbit_amplitude { orbit_amplitude } else { bail!("orbit_amplitude unset") };
    let orbit_period = if let Some(orbit_period) = orbit_period { orbit_period } else { bail!("orbit_period unset") };
    let orbit_center = if let Some(orbit_center) = orbit_center { orbit_center } else { bail!("orbit_center unset") };
    let orbit = OrbitPattern::new(orbit_amplitude, orbit_period, orbit_center);

    let seed = if let Some(seed) = seed { seed } else { bail!("seed unset") };

    let mut earth_map = None;
    let mut mars_map = None;
    for (i, planet) in planets.iter().enumerate() {
        let p = if i == 0 {
            Planet::Earth
        } else {
            Planet::Mars
        };
        let width = if let Some(width) = planet.width { width } else { bail!("width unset") };
        let height = if let Some(height) = planet.height { height } else { bail!("height unset") };
        let mut map = PlanetMap {
            planet: p,
            height: height,
            width: width,
            initial_units: vec![],
            is_passable_terrain: vec![vec![true; width]; height],
            initial_karbonite: vec![vec![0; width]; height],
        };

        let mut id = 1;

        for (&(x,y), thing) in planet.things.iter() {
            map.is_passable_terrain[y as usize][x as usize] = thing.passable;
            map.initial_karbonite[y as usize][x as usize] = thing.karbonite;

            if let Some(t) = thing.team {
                map.initial_units.push(Unit::new(
                    id,
                    t,
                    UnitType::Worker,
                    0,
                    Location::OnMap(MapLocation::new(p, x as i32, y as i32))
                ).unwrap());
                id += 1;
            }
        }
        if p == Planet::Earth {
            earth_map = Some(map);
        } else {
            mars_map = Some(map);
        }
    }
    let mut ast = FnvHashMap::default();
    for (round, x, y, k) in asteroids {
        ast.insert(round as Rounds, AsteroidStrike {
            karbonite: k as u32,
            location: MapLocation::new(Planet::Mars, x as i32, y as i32)
        });
    }

    Ok(GameMap {
        seed,
        earth_map: earth_map.unwrap(),
        mars_map: mars_map.unwrap(),
        asteroids: AsteroidPattern { pattern: ast },
        orbit
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_bananas() {
        let bananas = include_str!("bananas.bc18t");
        let map = parse_text_map(bananas).unwrap();

        assert_eq!(map.orbit.amplitude, 50);
        assert_eq!(map.orbit.period, 40);
        assert_eq!(map.orbit.center, 100);

        assert_eq!(map.asteroids.pattern[&200].karbonite, 150);
        assert_eq!(map.mars_map.initial_karbonite[29][29], 1000);

        let mut founda = false;
        let mut foundb = false;
        for unit in &map.earth_map.initial_units {
            if unit.location().map_location().unwrap().x == 2
                && unit.location().map_location().unwrap().y == 18
                && unit.team() == Team::Red
                && unit.unit_type() == UnitType::Worker {
                founda = true;
            }
            if unit.location().map_location().unwrap().x == 2
                && unit.location().map_location().unwrap().y == 1
                && unit.team() == Team::Blue
                && unit.unit_type() == UnitType::Worker {
                foundb = true;
            }

        }
        assert!(founda);
        assert!(foundb);
    }
    #[test]
    fn parse_fat() {
        let fat = include_str!("fat.bc18t");
        let map = parse_text_map(fat).unwrap();
    }
}
