#![allow(clippy::items_after_test_module)]
#![cfg(feature = "serde")]
use ghrepo::GHRepo;
use rstest::rstest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Repository {
    id: u64,
    name: GHRepo,
}

#[rstest]
#[case(r#"{"id": 12345, "name": "jwodder/ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "git://github.com/jwodder/ghrepo-rust"}"#)]
#[case(r#"{"name": "git@github.com:jwodder/ghrepo-rust.git", "id": 12345}"#)]
#[case(r#"{"id": 12345, "name": "ssh://git@github.com/jwodder/ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "https://api.github.com/repos/jwodder/ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "api.github.com/repos/jwodder/ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "https://github.com/jwodder/ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "https://github.com/jwodder/ghrepo-rust.git"}"#)]
fn test_deserialize_ghrepo(#[case] json: &str) {
    assert_eq!(
        serde_json::from_str::<Repository>(json).unwrap(),
        Repository {
            id: 12345,
            name: GHRepo::new("jwodder", "ghrepo-rust").unwrap()
        }
    );
}

#[rstest]
#[case(r#"{"id": 12345, "name": "ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "none/ghrepo-rust"}"#)]
#[case(r#"{"id": 12345, "name": "https://github.com/none/headerparser.git"}"#)]
#[case(r#"{"id": 12345, "name": "https://github.com/joe.coder/hello-world"}"#)]
#[case(r#"{"id": 12345, "name": "jwodder/headerparser.git"}"#)]
#[case(r#"{"id": 12345, "name": "https://api.github.com/repos/jwodder/headerparser.git"}"#)]
#[case(r#"{"id": 12345, "name": ""}"#)]
#[case(r#"{"id": 12345, "name": null}"#)]
#[case(r#"{"id": 12345, "name": 42}"#)]
#[case(r#"{"id": 12345, "name": true}"#)]
#[case(r#"{"id": 12345, "name": ["jwodder/ghrepo-rust"]}"#)]
#[case(r#"{"id": 12345, "name": ["jwodder", "ghrepo-rust"]}"#)]
#[case(r#"{"id": 12345, "name": {"jwodder": "ghrepo-rust"}}"#)]
fn test_deserialize_err(#[case] json: &str) {
    assert!(serde_json::from_str::<Repository>(json).is_err());
}

#[test]
fn test_serialize_ghrepo() {
    let r = Repository {
        id: 12345,
        name: GHRepo::new("jwodder", "ghrepo-rust").unwrap(),
    };
    assert_eq!(
        serde_json::to_string(&r).unwrap(),
        r#"{"id":12345,"name":"jwodder/ghrepo-rust"}"#
    );
}
