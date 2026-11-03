//! Pure AND filtering. Unknown numeric/status fields never satisfy a constraint.
use crate::model::{AuthMode, Health, Server, Sort, Status, Tab, View};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Version {
    #[default]
    Any,
    Compatible,
    Specific,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Filters {
    pub online: Option<bool>,
    pub version: Version,
    pub specific_version: String,
    pub min_players: Option<u32>,
    pub max_players: Option<u32>,
    pub free_slots: bool,
    pub not_empty: bool,
    pub not_full: bool,
    pub max_ping: Option<u64>,
    pub tags: Vec<String>,
    pub language: String,
    pub region: String,
    pub group: String,
    pub favorites: bool,
    pub auth: Option<AuthMode>,
    /// Conservative offline-mode-only filter, independent of account handling.
    pub joinable: bool,
}

pub fn strip_formatting(s: &str) -> String {
    let mut chars = s.chars();
    let mut result = String::with_capacity(s.len());
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            result.push(c);
        }
    }
    result
}

pub fn matches(server: &Server, status: Option<&Status>, view: &View) -> bool {
    let m = &server.metadata;
    let f = &view.filters;
    if (view.tab == Tab::Favorites || f.favorites) && !m.favorite
        || view.tab == Tab::Recent && m.last_played == 0
        || f.joinable && m.auth != AuthMode::Offline
        || f.auth.is_some_and(|auth| auth != m.auth)
        || !f.group.is_empty() && !f.group.eq_ignore_ascii_case(&m.group)
        || !f.language.is_empty() && !f.language.eq_ignore_ascii_case(&m.language)
        || !f.region.is_empty() && !f.region.eq_ignore_ascii_case(&m.region)
        || f.tags
            .iter()
            .any(|tag| !m.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
    {
        return false;
    }
    if f.online.is_some_and(|online| {
        status.is_none_or(|s| {
            if online {
                s.health != Health::Online
            } else {
                matches!(s.health, Health::Online | Health::Pinging)
            }
        })
    }) || f.version == Version::Compatible && status.is_none_or(|s| !s.compatible())
        || f.version == Version::Specific
            && status.is_none_or(|s| !s.version.eq_ignore_ascii_case(&f.specific_version))
        || f.min_players
            .is_some_and(|n| status.and_then(|s| s.online).is_none_or(|p| p < n))
        || f.max_players
            .is_some_and(|n| status.and_then(|s| s.online).is_none_or(|p| p > n))
        || f.not_empty && status.and_then(|s| s.online).is_none_or(|n| n == 0)
        || (f.free_slots || f.not_full)
            && status
                .and_then(|s| s.online.zip(s.max))
                .is_none_or(|(n, max)| n >= max)
        || f.max_ping
            .is_some_and(|n| status.and_then(|s| s.ms).is_none_or(|p| p > n))
    {
        return false;
    }
    if view.search.trim().is_empty() {
        return true;
    }
    let haystack = strip_formatting(&format!(
        "{} {} {} {} {} {}",
        server.name,
        server.address,
        status.map_or("", |s| s.motd.as_str()),
        m.tags.join(" "),
        m.notes,
        m.group
    ))
    .to_lowercase();
    let search_lower = view.search.to_lowercase();
    search_lower.split_whitespace().all(|word| {
        if let Some(tag) = word.strip_prefix('#') {
            m.tags.iter().any(|t| t.to_lowercase().contains(tag))
        } else if let Some(group) = word.strip_prefix('@') {
            m.group.to_lowercase().contains(group)
        } else {
            haystack.contains(word)
        }
    })
}

pub fn indices(servers: &[Server], statuses: &HashMap<String, Status>, view: &View) -> Vec<usize> {
    let status = |s: &Server| statuses.get(&s.address);
    let mut result: Vec<_> = servers
        .iter()
        .enumerate()
        .filter(|(_, s)| matches(s, status(s), view))
        .map(|(i, _)| i)
        .collect();
    result.sort_by(|&a, &b| {
        let (a_server, b_server) = (&servers[a], &servers[b]);
        let (a_status, b_status) = (status(a_server), status(b_server));
        // Unknown values sort last in BOTH directions; stable ties retain custom order.
        let numeric = |x: Option<u64>, y: Option<u64>| option_cmp(x, y, view.descending);
        let order = match view.sort {
            Sort::Custom => a.cmp(&b),
            Sort::Name => a_server
                .name
                .to_lowercase()
                .cmp(&b_server.name.to_lowercase()),
            Sort::Players => numeric(
                a_status.and_then(|s| s.online.map(u64::from)),
                b_status.and_then(|s| s.online.map(u64::from)),
            ),
            Sort::FreeSlots => numeric(
                a_status.and_then(|s| {
                    s.max
                        .zip(s.online)
                        .map(|(m, n)| u64::from(m.saturating_sub(n)))
                }),
                b_status.and_then(|s| {
                    s.max
                        .zip(s.online)
                        .map(|(m, n)| u64::from(m.saturating_sub(n)))
                }),
            ),
            Sort::Ping => numeric(a_status.and_then(|s| s.ms), b_status.and_then(|s| s.ms)),
            Sort::Version => version_cmp(
                a_status.map_or("", |s| &s.version),
                b_status.map_or("", |s| &s.version),
            ),
            Sort::LastPlayed => a_server
                .metadata
                .last_played
                .cmp(&b_server.metadata.last_played),
            Sort::DateAdded => a_server.metadata.added.cmp(&b_server.metadata.added),
            Sort::Favorites => b_server.metadata.favorite.cmp(&a_server.metadata.favorite),
        };
        let final_order = if matches!(view.sort, Sort::Players | Sort::FreeSlots | Sort::Ping) {
            order
        } else if view.descending {
            order.reverse()
        } else {
            order
        };
        if final_order == Ordering::Equal {
            a.cmp(&b)
        } else {
            final_order
        }
    });
    result
}

fn option_cmp(a: Option<u64>, b: Option<u64>, descending: bool) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => {
            if descending {
                b.cmp(&a)
            } else {
                a.cmp(&b)
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}
fn version_cmp(a: &str, b: &str) -> Ordering {
    let parts = |s: &str| {
        s.split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse::<u64>().ok())
            .collect::<Vec<_>>()
    };
    parts(a).cmp(&parts(b)).then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Server, Status) {
        let mut server = Server {
            name: "§aBerlin Craft".into(),
            address: "localhost".into(),
            ..Server::default()
        };
        server.metadata.tags = vec!["Survival".into(), "EU".into()];
        server.metadata.notes = "friends welcome".into();
        server.metadata.auth = AuthMode::Offline;
        server.metadata.favorite = true;
        server.metadata.language = "de".into();
        server.metadata.region = "DE".into();
        (
            server,
            Status {
                health: Health::Online,
                motd: "§bA peaceful world".into(),
                online: Some(5),
                max: Some(20),
                ms: Some(45),
                version: "26.3".into(),
                protocol: Some(horizon_protocol::PROTOCOL_VERSION),
                ..Status::default()
            },
        )
    }
    #[test]
    fn search_is_unicode_case_insensitive_multiword_and() {
        let (s, p) = fixture();
        for search in ["CRAFT friends", "peaceful survival", "BERLIN localhost"] {
            assert!(matches(
                &s,
                Some(&p),
                &View {
                    search: search.into(),
                    ..View::default()
                }
            ));
        }
        assert!(!matches(
            &s,
            Some(&p),
            &View {
                search: "friends missing".into(),
                ..View::default()
            }
        ));
        assert_eq!(strip_formatting("§aHi §lthere§r!"), "Hi there!");
    }
    #[test]
    fn every_filter_combines_and_unknown_is_not_joinable() {
        let (mut s, p) = fixture();
        let mut v = View {
            filters: Filters {
                online: Some(true),
                version: Version::Compatible,
                min_players: Some(2),
                max_players: Some(10),
                free_slots: true,
                not_empty: true,
                not_full: true,
                max_ping: Some(50),
                tags: vec!["eu".into()],
                language: "de".into(),
                region: "DE".into(),
                favorites: true,
                joinable: true,
                ..Filters::default()
            },
            ..View::default()
        };
        assert!(matches(&s, Some(&p), &v));
        assert!(!matches(&s, None, &v));
        s.metadata.auth = AuthMode::Unknown;
        assert!(!matches(&s, Some(&p), &v));
        s.metadata.auth = AuthMode::Offline;
        v.filters.min_players = Some(6);
        assert!(!matches(&s, Some(&p), &v));
        v.filters.min_players = None;
        v.filters.max_ping = Some(44);
        assert!(!matches(&s, Some(&p), &v));
    }
    #[test]
    fn sorts_are_stable_and_unknown_pings_stay_last_both_ways() {
        let (s, p) = fixture();
        let mut b = s.clone();
        b.address = "other".into();
        b.name = "Alpha".into();
        let servers = vec![s, b];
        let map = HashMap::from([("localhost".into(), p)]);
        for sort in Sort::ALL {
            for descending in [false, true] {
                let v = View {
                    sort,
                    descending,
                    ..View::default()
                };
                let result = indices(&servers, &map, &v);
                assert_eq!(result.len(), 2);
                assert_ne!(result[0], result[1]);
                if sort == Sort::Ping {
                    assert_eq!(result, vec![0, 1]);
                }
            }
        }
        assert_eq!(
            indices(
                &servers,
                &map,
                &View {
                    sort: Sort::Name,
                    ..View::default()
                }
            ),
            vec![1, 0]
        );
        assert_eq!(version_cmp("1.9", "1.20"), Ordering::Less);
    }
    #[test]
    fn ten_thousand_entries_are_filtered_and_stably_sorted() {
        let servers = (0..10_000)
            .map(|i| Server {
                name: format!("Server {i}"),
                address: format!("s{i}.example.org"),
                ..Server::default()
            })
            .collect::<Vec<_>>();
        let v = View {
            search: "server 999".into(),
            ..View::default()
        };
        let result = indices(&servers, &HashMap::new(), &v);
        assert_eq!(result.len(), 19);
        assert_eq!(result[0], 999);
    }
    #[test]
    fn each_constraint_rejects_nonmatching_and_unknown_values() {
        let (s, p) = fixture();
        let cases = [
            Filters {
                online: Some(false),
                ..Filters::default()
            },
            Filters {
                version: Version::Specific,
                specific_version: "1.9".into(),
                ..Filters::default()
            },
            Filters {
                min_players: Some(6),
                ..Filters::default()
            },
            Filters {
                max_players: Some(4),
                ..Filters::default()
            },
            Filters {
                max_ping: Some(44),
                ..Filters::default()
            },
            Filters {
                tags: vec!["missing".into()],
                ..Filters::default()
            },
            Filters {
                language: "en".into(),
                ..Filters::default()
            },
            Filters {
                region: "US".into(),
                ..Filters::default()
            },
            Filters {
                group: "other".into(),
                ..Filters::default()
            },
            Filters {
                auth: Some(AuthMode::Online),
                ..Filters::default()
            },
        ];
        for filters in cases {
            assert!(!matches(
                &s,
                Some(&p),
                &View {
                    filters,
                    ..View::default()
                }
            ));
        }
        for filters in [
            Filters {
                not_empty: true,
                ..Filters::default()
            },
            Filters {
                not_full: true,
                ..Filters::default()
            },
            Filters {
                free_slots: true,
                ..Filters::default()
            },
        ] {
            assert!(!matches(
                &s,
                None,
                &View {
                    filters: filters.clone(),
                    ..View::default()
                }
            ));
            let full = Status {
                online: Some(0),
                max: Some(0),
                ..p.clone()
            };
            assert!(!matches(
                &s,
                Some(&full),
                &View {
                    filters,
                    ..View::default()
                }
            ));
        }
        assert!(!matches(
            &s,
            Some(&Status::default()),
            &View {
                filters: Filters {
                    online: Some(false),
                    ..Filters::default()
                },
                ..View::default()
            }
        ));
        assert!(!matches(
            &s,
            Some(&p),
            &View {
                tab: Tab::Recent,
                ..View::default()
            }
        ));
        let mut normal = s.clone();
        normal.metadata.favorite = false;
        assert!(!matches(
            &normal,
            Some(&p),
            &View {
                tab: Tab::Favorites,
                ..View::default()
            }
        ));
    }
    #[test]
    fn every_sort_has_the_expected_order_in_both_directions() {
        let (mut a, mut ap) = fixture();
        let (mut b, mut bp) = fixture();
        a.name = "Zebra".into();
        b.name = "Alpha".into();
        b.address = "other".into();
        a.metadata.last_played = 20;
        b.metadata.last_played = 10;
        a.metadata.added = 20;
        b.metadata.added = 10;
        b.metadata.favorite = false;
        ap.online = Some(8);
        ap.max = Some(20);
        ap.ms = Some(40);
        ap.version = "1.20".into();
        bp.online = Some(2);
        bp.max = Some(3);
        bp.ms = Some(10);
        bp.version = "1.9".into();
        let map = HashMap::from([(a.address.clone(), ap), (b.address.clone(), bp)]);
        let servers = vec![a, b];
        for sort in Sort::ALL {
            let expected = if matches!(sort, Sort::Custom | Sort::Favorites) {
                vec![0, 1]
            } else {
                vec![1, 0]
            };
            assert_eq!(
                indices(
                    &servers,
                    &map,
                    &View {
                        sort,
                        ..View::default()
                    }
                ),
                expected,
                "{sort:?}"
            );
            let mut reversed = expected;
            reversed.reverse();
            assert_eq!(
                indices(
                    &servers,
                    &map,
                    &View {
                        sort,
                        descending: true,
                        ..View::default()
                    }
                ),
                reversed,
                "{sort:?}"
            );
        }
    }
    #[test]
    fn tightening_minimum_players_only_removes_results() {
        let (s, p) = fixture();
        for online in 0..100 {
            let p = Status {
                online: Some(online),
                ..p.clone()
            };
            let mut previous = true;
            for minimum in 0..110 {
                let result = matches(
                    &s,
                    Some(&p),
                    &View {
                        filters: Filters {
                            min_players: Some(minimum),
                            ..Filters::default()
                        },
                        ..View::default()
                    },
                );
                assert!(!result || previous);
                previous = result;
            }
        }
    }
}
