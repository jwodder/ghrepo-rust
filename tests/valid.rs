#![allow(clippy::items_after_test_module)]
use ghrepo::{is_valid_name, is_valid_owner};
use rstest::rstest;

#[rstest]
#[case("steven-universe")]
#[case("steven")]
#[case("s")]
#[case("s-u")]
#[case("7152")]
#[case("s-t-e-v-e-n")]
#[case("s-t-eeeeee-v-e-n")]
#[case("peridot-2F5L-5XG")]
#[case("nonely")]
#[case("none-one")]
#[case("none-none")]
#[case("nonenone")]
#[case("none0")]
#[case("0none")]
// The following are actual usernames on GitHub that violate the current
// username restrictions:
#[case("-")]
#[case("-Jerry-")]
#[case("-SFT-Clan")]
#[case("123456----")]
#[case("FirE-Fly-")]
#[case("None-")]
#[case("alex--evil")]
#[case("johan--")]
#[case("pj_nitin")]
#[case("up_the_irons")]
fn test_good_owner(#[case] owner: &str) {
    assert!(is_valid_owner(owner));
}

#[rstest]
#[case("steven.universe")]
#[case("steven-universe@beachcity.dv")]
#[case("steven-univerß")]
#[case("")]
#[case("none")]
#[case("NONE")]
fn test_bad_owner(#[case] owner: &str) {
    assert!(!is_valid_owner(owner));
}

#[rstest]
#[case("steven-universe")]
#[case("steven")]
#[case("s")]
#[case("s-u")]
#[case("7152")]
#[case("s-t-e-v-e-n")]
#[case("s-t-eeeeee-v-e-n")]
#[case("peridot-2F5L-5XG")]
#[case("...")]
#[case("-steven")]
#[case("steven-")]
#[case("-steven-")]
#[case("steven.universe")]
#[case("steven_universe")]
#[case("steven--universe")]
#[case("s--u")]
#[case("git.steven")]
#[case("steven.git.txt")]
#[case("steven.gitt")]
#[case(".gitt")]
#[case("..gitt")]
#[case("...gitt")]
#[case("git")]
#[case("-")]
#[case("_")]
#[case("---")]
#[case(".---")]
#[case(".steven")]
#[case("..steven")]
#[case("...steven")]
fn test_good_name(#[case] name: &str) {
    assert!(is_valid_name(name));
}

#[rstest]
#[case("steven-univerß")]
#[case(".")]
#[case("..")]
#[case("...git")]
#[case("..git")]
#[case(".git")]
#[case("")]
#[case("steven.git")]
#[case("steven.GIT")]
#[case("steven.Git")]
fn test_bad_name(#[case] name: &str) {
    assert!(!is_valid_name(name));
}
