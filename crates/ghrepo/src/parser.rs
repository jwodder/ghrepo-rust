use std::borrow::Cow;

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

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    /// A string to match exactly
    Literal(Cow<'static, str>),
    /// A string to match regardless of differences in ASCII case
    CaseFold(Cow<'static, str>),
}

impl From<&'static str> for Token {
    fn from(s: &'static str) -> Token {
        Token::Literal(Cow::Borrowed(s))
    }
}

impl From<String> for Token {
    fn from(s: String) -> Token {
        Token::Literal(Cow::Owned(s))
    }
}

fn start_patterns() -> Vec<(Vec<Token>, State)> {
    let mut patterns = vec![
        (vec![Token::CaseFold("https://".into())], State::Http),
        (vec![Token::CaseFold("http://".into())], State::Http),
        (
            vec![
                Token::CaseFold("api.github.com".into()),
                Token::Literal("/repos/".into()),
            ],
            State::OwnerName,
        ),
        (
            vec![Token::CaseFold("git://github.com/".into())],
            State::OwnerNameGit,
        ),
        (
            vec![
                Token::Literal("git@".into()),
                Token::CaseFold("github.com:".into()),
            ],
            State::OwnerNameGit,
        ),
        (
            vec![
                Token::CaseFold("ssh://".into()),
                Token::Literal("git@".into()),
                Token::CaseFold("github.com/".into()),
            ],
            State::OwnerNameGit,
        ),
    ];

    // Add GH_HOST / Enterprise patterns if it's set and different from github.com
    //
    // GH_HOST is a valid environment variable for the 'gh' command.
    if let Ok(host) = std::env::var("GH_HOST") {
        if host != "github.com" {
            let api_host = format!("api.{host}");
            let git_url = format!("git://{host}/");
            let ssh_host = format!("{host}:");
            let ssh_url = format!("{host}/");

            patterns.extend(vec![
                (
                    vec![
                        Token::CaseFold(api_host.into()),
                        Token::Literal("/repos/".into()),
                    ],
                    State::OwnerName,
                ),
                (vec![Token::CaseFold(git_url.into())], State::OwnerNameGit),
                (
                    vec![
                        Token::Literal("git@".into()),
                        Token::CaseFold(ssh_host.into()),
                    ],
                    State::OwnerNameGit,
                ),
                (
                    vec![
                        Token::CaseFold("ssh://".into()),
                        Token::Literal("git@".into()),
                        Token::CaseFold(ssh_url.into()),
                    ],
                    State::OwnerNameGit,
                ),
            ]);
        }
    }

    patterns
}

/// If `s` is a valid GitHub repository URL, return the repository owner &
/// name.  The following URL formats are recognized:
///
/// - `[http[s]://[<username>[:<password>]@]][www.]github.com/<owner>/<name>[.git][/]`
/// - `[http[s]://]api.github.com/repos/<owner>/<name>`
/// - `git://github.com/<owner>/<name>[.git]`
/// - `git@github.com:<owner>/<name>[.git]`
/// - `ssh://git@github.com/<owner>/<name>[.git]`
pub(crate) fn parse_github_url(s: &str) -> Option<(&str, &str)> {
    // Notes on case sensitivity:
    // - Schemes & hostnames in URLs are case insensitive per RFC 3986 (though
    //   `git clone` as of Git 2.38.1 doesn't actually accept non-lowercase
    //   schemes).
    // - The "repos" in an API URL is case sensitive; changing the case results
    //   in a 404.
    // - The "git" username in SSH URLs (both forms) is case sensitive;
    //   changing the case results in a permissions error.
    // - The optional ".git" suffix is case sensitive; changing the case (when
    //   cloning with `git clone`, at least) results in either a credentials
    //   prompt for HTTPS URLs (the same as if you'd specified a nonexistent
    //   repo) or a "repository not found" message for SSH URLs.
    let start_patterns = start_patterns();

    let mut parser = PullParser::new(s);
    let mut state = State::Start;
    let mut result: Option<(&str, &str)> = None;
    loop {
        state = match state {
            State::Start => start_patterns
                .iter()
                .find_map(|(tokens, transition)| parser.consume_seq(tokens).and(Some(*transition)))
                .unwrap_or(State::Web),
            State::Http => {
                if parser
                    .consume_seq(&[
                        Token::CaseFold("api.github.com".into()),
                        Token::Literal("/repos/".into()),
                    ])
                    .is_some()
                {
                    State::OwnerName
                } else if let Ok(host) = std::env::var("GH_HOST") {
                    if host != "github.com" {
                        let api_host = format!("api.{host}");

                        if parser
                            .consume_seq(&[
                                Token::CaseFold(api_host.into()),
                                Token::Literal("/repos/".into()),
                            ])
                            .is_some()
                        {
                            State::OwnerName
                        } else {
                            parser.maybe_consume_userinfo();
                            State::Web
                        }
                    } else {
                        parser.maybe_consume_userinfo();
                        State::Web
                    }
                } else {
                    parser.maybe_consume_userinfo();
                    State::Web
                }
            }
            State::Web => {
                parser.maybe_consume(Token::CaseFold("www.".into()));

                let mut hosts_to_try = vec!["github.com".to_string()];

                if let Ok(host) = std::env::var("GH_HOST") {
                    if host != "github.com" {
                        hosts_to_try.push(host);
                    }
                }

                // Try each host
                for host in hosts_to_try {
                    let web_host = format!("{host}/");

                    if parser.consume(Token::CaseFold(web_host.into())).is_some() {
                        result = Some(parser.get_owner_name()?);
                        parser.maybe_consume(".git".into());
                        parser.maybe_consume("/".into());
                        return if parser.at_end() { result } else { None };
                    }
                }

                return None;
            }
            State::OwnerName => {
                result = Some(parser.get_owner_name()?);
                State::End
            }
            State::OwnerNameGit => {
                result = Some(parser.get_owner_name()?);
                parser.maybe_consume(".git".into());
                State::End
            }
            State::End => return if parser.at_end() { result } else { None },
        }
    }
}

struct PullParser<'a> {
    data: &'a str,
}

impl<'a> PullParser<'a> {
    fn new(data: &'a str) -> Self {
        Self { data }
    }

    fn consume_seq<'b, I>(&mut self, tokens: I) -> Option<()>
    where
        I: IntoIterator<Item = &'b Token>,
    {
        let orig = self.data;
        for t in tokens {
            if self.consume(t.clone()).is_none() {
                self.data = orig;
                return None;
            }
        }
        Some(())
    }

    fn consume(&mut self, token: Token) -> Option<()> {
        match token {
            Token::Literal(s) => match self.data.strip_prefix(s.as_ref()) {
                Some(t) => {
                    self.data = t;
                    Some(())
                }
                None => None,
            },
            Token::CaseFold(s) => {
                let i = s.len();
                match self.data.get(..i).zip(self.data.get(i..)) {
                    Some((t, u)) if t.eq_ignore_ascii_case(s.as_ref()) => {
                        self.data = u;
                        Some(())
                    }
                    _ => None,
                }
            }
        }
    }

    fn maybe_consume(&mut self, token: Token) {
        let _ = self.consume(token);
    }

    fn get_owner_name(&mut self) -> Option<(&'a str, &'a str)> {
        let (owner, name, s) = split_owner_name(self.data)?;
        self.data = s;
        Some((owner, name))
    }

    /// If the current state starts with a (possibly empty) URL userinfo field
    /// followed by a `@`, consume them both.
    fn maybe_consume_userinfo(&mut self) {
        // cf. <https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1>
        if let Some((userinfo, s)) = self.data.split_once('@') {
            if userinfo.chars().all(is_userinfo_char) {
                self.data = s;
            }
        }
    }

    fn at_end(&self) -> bool {
        self.data.is_empty()
    }
}

fn is_userinfo_char(c: char) -> bool {
    // RFC 3986 requires that percent signs be followed by two hex digits, but
    // we're not going to bother enforcing that.
    c.is_ascii_alphanumeric() || "-._~!$&'()*+,;=%:".contains(c)
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
    #[case("foobar", "foo".into(), Some(()), "bar")]
    #[case("FOObar", "foo".into(), None, "FOObar")]
    #[case("FOObar", Token::CaseFold("foo".into()), Some(()), "bar")]
    #[case("Pokémon", Token::CaseFold("poké".into()), Some(()), "mon")]
    #[case("PokÉmon", Token::CaseFold("poké".into()), None, "PokÉmon")]
    #[case("Pokémon", Token::CaseFold("poke".into()), None, "Pokémon")]
    #[case("foo", Token::CaseFold("foobar".into()), None, "foo")]
    #[case("foo", Token::CaseFold("FOO".into()), Some(()), "")]
    fn test_consume(
        #[case] start: &str,
        #[case] token: Token,
        #[case] out: Option<()>,
        #[case] end: &str,
    ) {
        let mut parser = PullParser::new(start);
        assert_eq!(parser.consume(token), out);
        assert_eq!(parser.data, end);
    }

    #[rstest]
    #[case("FOOBar", &[Token::CaseFold("foo".into()), "bar".into()], None, "FOOBar")]
    #[case("FOOBar", &[Token::CaseFold("foo".into()), Token::CaseFold("bar".into())], Some(()), "")]
    fn test_consume_seq(
        #[case] start: &str,
        #[case] tokens: &[Token],
        #[case] out: Option<()>,
        #[case] end: &str,
    ) {
        let mut parser = PullParser::new(start);
        assert_eq!(parser.consume_seq(tokens), out);
        assert_eq!(parser.data, end);
    }

    #[rstest]
    #[case("git://github.com/jwodder/headerparser", Some(("jwodder", "headerparser")))]
    #[case("GIT://GitHub.COM/jwodder/headerparser", Some(("jwodder", "headerparser")))]
    #[case("git@github.com:joe-q-coder/my.repo.git", Some(("joe-q-coder", "my.repo")))]
    #[case("git@GITHUB.com:joe-q-coder/my.repo.git", Some(("joe-q-coder", "my.repo")))]
    #[case("git@GITHUB.com:joe-q-coder/my.repo.GIT", None)]
    #[case("GIT@github.com:joe-q-coder/my.repo.git", None)]
    #[case("git@github.com/joe-q-coder/my.repo.git", None)]
    #[case("https://github.com/joe.coder/hello-world", None)]
    #[case("https://github.com/joe-coder/hello.world", Some(("joe-coder", "hello.world")))]
    #[case("http://github.com/joe-coder/hello.world", Some(("joe-coder", "hello.world")))]
    #[case("HTTPS://GITHUB.COM/joe-coder/hello.world", Some(("joe-coder", "hello.world")))]
    #[case("HTTPS://WWW.GITHUB.COM/joe-coder/hello.world", Some(("joe-coder", "hello.world")))]
    #[case("ssh://git@github.com/-/test", Some(("-", "test")))]
    #[case("SSH://git@GITHUB.COM/-/test", Some(("-", "test")))]
    #[case("SSH://Git@GITHUB.COM/-/test", None)]
    #[case("ssh://GIT@github.com/-/test", None)]
    #[case("https://api.github.com/repos/none-/-none", Some(("none-", "-none")))]
    #[case("HttpS://api.github.com/repos/none-/-none", Some(("none-", "-none")))]
    #[case("http://api.github.com/repos/none-/-none", Some(("none-", "-none")))]
    #[case("Http://api.github.com/repos/none-/-none", Some(("none-", "-none")))]
    #[case("api.github.com/repos/jwodder/headerparser", Some(("jwodder", "headerparser")))]
    #[case("api.github.com/REPOS/jwodder/headerparser", None)]
    #[case("Api.GitHub.Com/repos/jwodder/headerparser", Some(("jwodder", "headerparser")))]
    #[case("https://github.com/-Jerry-/geshi-1.0.git", Some(("-Jerry-", "geshi-1.0")))]
    #[case("https://github.com/-Jerry-/geshi-1.0.Git", None)]
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
    #[case("ssh://git@github.com:jwodder/headerparser", None)]
    #[case("ssh://git@github.com:jwodder/headerparser/", None)]
    #[case("ssh://git@github.com/jwodder/headerparser/", None)]
    #[case("git://github.com/jwodder/headerparser/", None)]
    #[case("https://http://github.com/joe-coder/hello.world", None)]
    #[case(
        "https://x-access-token:1234567890@api.github.com/repos/octocat/Hello-World",
        None
    )]
    #[case("x-access-token:1234567890@github.com/octocat/Hello-World", None)]
    #[case("https://user name@github.com/octocat/Hello-World", None)]
    #[case("https://user/name@github.com/octocat/Hello-World", None)]
    #[case("https://user@name@github.com/octocat/Hello-World", None)]
    #[case("https://user%20name@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://user+name@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://~user.name@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://user:@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://:pass@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://:@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    #[case("https://user:pass:extra@github.com/octocat/Hello-World", Some(("octocat", "Hello-World")))]
    fn test_parse_github_url(#[case] s: &str, #[case] out: Option<(&str, &str)>) {
        assert_eq!(parse_github_url(s), out);
    }

    #[test]
    fn test_both_hosts_work_simultaneously() {
        // Save original GH_HOST value if it exists
        let original_gh_host = std::env::var("GH_HOST").ok();

        // SAFETY: Test with custom GH_HOST - should work for BOTH hosts
        unsafe { std::env::set_var("GH_HOST", "github.example.com") };

        // GitHub.com URLs should still work
        assert_eq!(
            parse_github_url("git@github.com:joe-q-coder/my.repo.git"),
            Some(("joe-q-coder", "my.repo"))
        );
        assert_eq!(
            parse_github_url("https://github.com/joe-coder/hello.world"),
            Some(("joe-coder", "hello.world"))
        );
        assert_eq!(
            parse_github_url("https://api.github.com/repos/jwodder/headerparser"),
            Some(("jwodder", "headerparser"))
        );

        // GH_HOST URLs should also work
        assert_eq!(
            parse_github_url("git@github.example.com:joe-q-coder/my.repo.git"),
            Some(("joe-q-coder", "my.repo"))
        );
        assert_eq!(
            parse_github_url("https://github.example.com/joe-coder/hello.world"),
            Some(("joe-coder", "hello.world"))
        );
        assert_eq!(
            parse_github_url("https://api.github.example.com/repos/jwodder/headerparser"),
            Some(("jwodder", "headerparser"))
        );

        // SAFETY: Restore original GH_HOST value
        unsafe {
            match original_gh_host {
                Some(val) => std::env::set_var("GH_HOST", val),
                None => std::env::remove_var("GH_HOST"),
            }
        };
    }
}
