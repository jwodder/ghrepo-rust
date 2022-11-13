/// Split a string into a maximal prefix of chars that match `pred` and the
/// remainder of the string
fn span<P>(s: &str, mut pred: P) -> (&str, &str)
where
    P: FnMut(char) -> bool,
{
    match s.find(|c| !pred(c)) {
        Some(i) => s.split_at(i),
        None => s.split_at(s.len()),
    }
}

/// If `s` starts with a valid GitHub owner (i.e., user or organization) name,
/// return the owner and the remainder of `s`.
pub(crate) fn split_owner(s: &str) -> Option<(&str, &str)> {
    let (owner, rem) = span(s, is_owner_char);
    if owner.is_empty() || owner.eq_ignore_ascii_case("none") {
        None
    } else {
        Some((owner, rem))
    }
}

fn is_owner_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// If `s` starts with a valid GitHub repository name, return the name and the
/// remainder of `s`.
pub(crate) fn split_name(s: &str) -> Option<(&str, &str)> {
    let (name, rem) = span(s, is_name_char);
    let (name, rem) = match name.len().checked_sub(4) {
        Some(i) if name.get(i..).unwrap_or("").eq_ignore_ascii_case(".git") => s.split_at(i),
        _ => (name, rem),
    };
    if name.is_empty() || name == "." || name == ".." {
        None
    } else {
        Some((name, rem))
    }
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
}

/// If `s` starts with a prefix of the form `OWNER/NAME`, where `OWNER` is a
/// valid GitHub owner and `NAME` is a valid GitHub repository name, return the
/// owner, the name, and the remainder of `s`.
pub(crate) fn split_owner_name(s: &str) -> Option<(&str, &str, &str)> {
    let (owner, s) = split_owner(s)?;
    let s = s.strip_prefix('/')?;
    let (name, s) = split_name(s)?;
    Some((owner, name, s))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    Start,
    Http,
    Web,
    OwnerName,
    OwnerNameGit,
    End,
}

/// If `s` is a valid GitHub repository URL, return the repository owner &
/// name.  The following URL formats are recognized:
///
/// - `[http[s]://[<username>[:<password>]@]][www.]github.com/<owner>/<name>[.git][/]`
/// - `[http[s]://]api.github.com/repos/<owner>/<name>`
/// - `git://github.com/<owner>/<name>[.git]`
/// - `[ssh://]git@github.com:<owner>/<name>[.git]`
pub(crate) fn parse_github_url(mut s: &str) -> Option<(&str, &str)> {
    let mut state = State::Start;
    let mut result: Option<(&str, &str)> = None;
    loop {
        (s, state) = match state {
            State::Start => [
                ("https://", State::Http),
                ("http://", State::Http),
                ("api.github.com/repos/", State::OwnerName),
                ("git://github.com/", State::OwnerNameGit),
                ("git@github.com:", State::OwnerNameGit),
                ("ssh://git@github.com:", State::OwnerNameGit),
            ]
            .into_iter()
            .find_map(|(token, transition)| s.strip_prefix(token).map(|t| (t, transition)))
            .unwrap_or((s, State::Web)),
            State::Http => match s.strip_prefix("api.github.com/repos/") {
                Some(t) => (t, State::OwnerName),
                None => (strip_user_pass(s), State::Web),
            },
            State::Web => {
                s = strip_optional_prefix(s, "www.");
                s = s.strip_prefix("github.com/")?;
                let (owner, name, mut t) = split_owner_name(s)?;
                result = Some((owner, name));
                t = strip_optional_prefix(t, ".git");
                t = strip_optional_prefix(t, "/");
                (t, State::End)
            }
            State::OwnerName => {
                let (owner, name, t) = split_owner_name(s)?;
                result = Some((owner, name));
                (t, State::End)
            }
            State::OwnerNameGit => {
                let (owner, name, t) = split_owner_name(s)?;
                result = Some((owner, name));
                (strip_optional_prefix(t, ".git"), State::End)
            }
            State::End => return if s.is_empty() { result } else { None },
        }
    }
}

/// If `s` starts with a prefix of the form "`username@`" or
/// "`username:password@`", return the part after the prefix; otherwise, return
/// `s` unchanged.
fn strip_user_pass(s: &str) -> &str {
    // TODO: Compare against <https://datatracker.ietf.org/doc/html/rfc3986>
    // (In particular, can the username or password be empty?)
    let (username, rem) = span(s, is_userpass_char);
    if username.is_empty() {
        return s;
    }
    let rem = match rem.strip_prefix(':') {
        Some(rem) => {
            let (password, rem) = span(rem, is_userpass_char);
            if password.is_empty() {
                return s;
            }
            rem
        }
        None => rem,
    };
    rem.strip_prefix('@').unwrap_or(s)
}

fn is_userpass_char(c: char) -> bool {
    c != '@' && c != ':' && c != '/'
}

/// If `s` starts with `prefix`, return the remainder of `s`; otherwise, return
/// `s` unchanged.
fn strip_optional_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    s.strip_prefix(prefix).unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("jwodder/ghrepo", Some(("jwodder", "/ghrepo")))]
    #[case("jwodder", Some(("jwodder", "")))]
    #[case("none/ghrepo", None)]
    #[case("NONE/ghrepo", None)]
    #[case("None/ghrepo", None)]
    #[case("nonely/ghrepo", Some(("nonely", "/ghrepo")))]
    #[case("", None)]
    #[case("/ghrepo", None)]
    fn test_split_owner(#[case] s: &str, #[case] out: Option<(&str, &str)>) {
        assert_eq!(split_owner(s), out);
    }

    #[rstest]
    #[case("ghrepo", Some(("ghrepo", "")))]
    #[case("ghrepo-rust", Some(("ghrepo-rust", "")))]
    #[case("gh.repo_rust", Some(("gh.repo_rust", "")))]
    #[case("ghrepo=good", Some(("ghrepo", "=good")))]
    #[case("ghrepo.git", Some(("ghrepo", ".git")))]
    #[case("ghrepo.GIT", Some(("ghrepo", ".GIT")))]
    #[case("ghrepo.Git", Some(("ghrepo", ".Git")))]
    #[case("ghrepo.git=good", Some(("ghrepo", ".git=good")))]
    #[case("", None)]
    #[case(".", None)]
    #[case("..", None)]
    #[case(".git", None)]
    #[case("..git", None)]
    #[case("...git", None)]
    #[case(".ghrepo.git", Some((".ghrepo", ".git")))]
    #[case("..ghrepo.git", Some(("..ghrepo", ".git")))]
    #[case("/ghrepo", None)]
    fn test_split_name(#[case] s: &str, #[case] out: Option<(&str, &str)>) {
        assert_eq!(split_name(s), out);
    }

    #[rstest]
    #[case("jwodder/ghrepo", Some(("jwodder", "ghrepo", "")))]
    #[case("jwodder/ghrepo.git", Some(("jwodder", "ghrepo", ".git")))]
    #[case("jwodder/ghrepo.git/", Some(("jwodder", "ghrepo", ".git/")))]
    #[case("jwodder/ghrepo/", Some(("jwodder", "ghrepo", "/")))]
    #[case("jwodder/ghrepo-rust", Some(("jwodder", "ghrepo-rust", "")))]
    #[case("jwodder/gh.repo_rust", Some(("jwodder", "gh.repo_rust", "")))]
    #[case("jwodder//ghrepo", None)]
    #[case("jwodder ghrepo", None)]
    #[case("none/ghrepo", None)]
    #[case("nonely/ghrepo", Some(("nonely", "ghrepo", "")))]
    fn test_split_owner_name(#[case] s: &str, #[case] out: Option<(&str, &str, &str)>) {
        assert_eq!(split_owner_name(s), out);
    }

    #[rstest]
    #[case("git://github.com/jwodder/headerparser", Some(("jwodder", "headerparser")))]
    #[case("git@github.com:joe-q-coder/my.repo.git", Some(("joe-q-coder", "my.repo")))]
    #[case("https://github.com/joe.coder/hello-world", None)]
    #[case("https://github.com/joe-coder/hello.world", Some(("joe-coder", "hello.world")))]
    #[case("ssh://git@github.com:-/test", Some(("-", "test")))]
    #[case("https://api.github.com/repos/none-/-none", Some(("none-", "-none")))]
    #[case("api.github.com/repos/jwodder/headerparser", Some(("jwodder", "headerparser")))]
    #[case("https://github.com/-Jerry-/geshi-1.0.git", Some(("-Jerry-", "geshi-1.0")))]
    #[case("https://github.com/-Jerry-/geshi-1.0.git/", Some(("-Jerry-", "geshi-1.0")))]
    #[case("https://github.com/-Jerry-/geshi-1.0/", Some(("-Jerry-", "geshi-1.0")))]
    #[case("https://www.github.com/-Jerry-/geshi-1.0", Some(("-Jerry-", "geshi-1.0")))]
    #[case("github.com/-Jerry-/geshi-1.0", Some(("-Jerry-", "geshi-1.0")))]
    #[case("https://x-access-token:1234567890@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://my.username@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://github.com/none/headerparser.git", None)]
    #[case("https://api.github.com/repos/jwodder/headerparser.git", None)]
    #[case("https://api.github.com/repos/jwodder/headerparser/", None)]
    #[case("my.username@www.github.com/octocat/Hello-World", None)]
    #[case("my.username:hunter2@github.com/octocat/Hello-World", None)]
    #[case("ssh://git@github.com:jwodder/headerparser/", None)]
    #[case("git://github.com/jwodder/headerparser/", None)]
    fn test_parse_github_url(#[case] s: &str, #[case] out: Option<(&str, &str)>) {
        assert_eq!(parse_github_url(s), out);
    }
}
