#![feature(let_chains)]

//! A toy of how bidding might work when running multiple trains on a single track.

use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::{BTreeMap, VecDeque};
use std::fmt;

/// There are 3 companies.
const COMPANIES: u32 = 2;
/// There are 5 sections.
const SECTIONS: u32 = 14;
/// The number of incremenets in the time period (e.g. this might be 1 week).
const TIME: u32 = 300;
const TIME_DISPLAY: u32 = 10;
/// Assuming circular track:
/// ```text
/// | --0-- --1-- | --2-- --3-- | --4-- --5-- | --6-- --7-- | --8-- --9-- | -12-- -13-- | --0-- --1-- |
/// |  Station A  |             |  Station B  |             |  Station C  |             |  Station A  |
/// ```

fn main() {
    // Company 0 wants to run a train from stations A->A. The size of the train is 2, it will wait 32 time units at each station and travels at 1 location unit/s each time unit
    let bid = generate_bid(
        0,
        0,
        0,
        0,
        &[(0, 32), (4, 32), (8, 32), (0, 32)],
        1,
        2,
        1024,
    );
    println!("{bid}");

    // Company 1 wants to run a train from stations A->C. The size of the train is 3, it will wait 32 time units at each station and travels at 2 location unit/s each time unit
    let bid = generate_bid(0, 64, 0, 8, &[(0, 32), (4, 32), (8, 32)], 1, 3, 2048);
    println!("{bid}");

    let allocations = resolve_bids(&[bid]);
    println!("{allocations}");
}

/// In working with a non-circular track a pathfinding algorithm will be required.
fn generate_bid(
    company: u32,
    start_time: u32,
    start_location: u32,
    end_location: u32,
    stops: &[(u32, u32)],
    speed: u32,
    size: u32,
    amount: u32,
) -> Bid {
    let mut ordered_stops = BTreeMap::<u32, VecDeque<u32>>::new();
    for (location, time) in stops {
        if let Some(s) = ordered_stops.get_mut(location) {
            s.push_back(*time);
        } else {
            let mut queue = VecDeque::new();
            queue.push_back(*time);
            ordered_stops.insert(*location, queue);
        }
    }

    let mut current_time = start_time;
    let mut current_location = start_location;
    let mut sections = Vec::new();
    loop {
        // println!("{current_time}, {current_location}");
        let location_range = current_location..current_location + size;
        let mut time_range = current_time..current_time + 1;

        if let Some(stop) = ordered_stops.get_mut(&current_location)
            && let Some(wait) = stop.pop_front()
        {
            time_range.end += wait;
            current_time += wait;
            if stop.is_empty() {
                ordered_stops.remove(&current_location);
            }
        }

        sections.extend(
            location_range
                .cartesian_product(time_range)
                .map(|(location, time)| Section { location, time }),
        );

        if current_location == end_location && ordered_stops.is_empty() {
            break;
        }

        current_time += 1;
        current_location += speed;
        if current_location >= SECTIONS {
            current_location %= SECTIONS;
        }
    }
    assert!(current_time < TIME);
    Bid {
        company,
        sections,
        amount,
    }
}

/// Given a number of bids, return the set of locations and times that are allocated to each company.
///
/// The returned locations and times should fulfil bids such that the `Bid::amount` is maximized.
fn resolve_bids(bids: &[Bid]) -> Allocation {
    todo!()
}

struct Allocation(BTreeMap<Section, u32>);
impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        assert!(COMPANIES < 10);

        let space = SECTIONS.ilog10();
        let mut string = String::new();
        for l in 0..SECTIONS {
            let log = if l == 0 { 0 } else { l.ilog10() };
            string.push_str(&format!("{}{l}│", " ".repeat((space - log) as usize)));

            for t in 0..TIME {
                if let Some(company) = self.0.get(&Section {
                    location: l,
                    time: t,
                }) {
                    string.push_str(&format!("{company}"));
                } else {
                    string.push(' ');
                }
            }
            string.push('\n');
        }

        string.push_str(&" ".repeat(space as usize + 1).to_string());
        string.push('└');
        for _ in 1..TIME {
            string.push('─');
        }
        string.push('\n');
        string.push_str(&" ".repeat(space as usize + 2).to_string());
        let mut over = 0;
        for t in 0..TIME {
            if t % TIME_DISPLAY == 0 {
                over = if t == 0 { 0 } else { t.ilog10() };
                string.push_str(&format!("{t}"));
            } else if over == 0 {
                string.push(' ');
            } else {
                over -= 1;
            }
        }

        write!(f, "{string}")
        // todo!()
    }
}

#[derive(Debug)]
struct Bid {
    /// The company the bid belongs to.
    company: u32,
    /// The sections the bid relates to.
    sections: Vec<Section>,
    /// The total amount for the bid.
    amount: u32,
}

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let space = SECTIONS.ilog10();
        let mut ordered_sections = BTreeMap::<u32, BTreeSet<u32>>::new();
        for Section { time, location } in &self.sections {
            if let Some(times) = ordered_sections.get_mut(location) {
                times.insert(*time);
            } else {
                let mut times = BTreeSet::new();
                times.insert(*time);
                ordered_sections.insert(*location, times);
            }
        }
        let mut string = String::new();
        for l in 0..SECTIONS {
            let log = if l == 0 { 0 } else { l.ilog10() };
            string.push_str(&format!("{}{l}│", " ".repeat((space - log) as usize)));

            if let Some(sections) = ordered_sections.get_mut(&l) {
                for t in 0..TIME {
                    if let Some(next) = sections.first() {
                        if *next == t {
                            string.push('*');
                            sections.pop_first();
                        } else {
                            string.push(' ');
                        }
                    } else {
                        break;
                    }
                }
            }
            string.push('\n');
            // string.push(' ');
        }

        string.push_str(&" ".repeat(space as usize + 1).to_string());
        string.push('└');
        for _ in 1..TIME {
            string.push('─');
        }
        string.push('\n');
        string.push_str(&" ".repeat(space as usize + 2).to_string());
        let mut over = 0;
        for t in 0..TIME {
            if t % TIME_DISPLAY == 0 {
                over = if t == 0 { 0 } else { t.ilog10() };
                string.push_str(&format!("{t}"));
            } else if over == 0 {
                string.push(' ');
            } else {
                over -= 1;
            }
        }

        write!(f, "{string}")
        // todo!()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Section {
    /// The location of the section of rail
    location: u32,
    /// The time slot within the upcoming period
    time: u32,
}

impl Ord for Section {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.location.cmp(&other.location) {
            Ordering::Equal => self.time.cmp(&other.time),
            ord => ord,
        }
    }
}

impl PartialOrd for Section {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
