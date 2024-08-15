use std::{borrow::Cow, fmt::Display};

use linkify::{LinkFinder, LinkKind};
use url::Url;

mod reddit;
use reddit::{alternative_reddit_links, resolve_reddit_share_link};

pub enum Link {
    Simple(String),
    Embed(String),
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Link::Simple(url) => write!(f, "<{}>", url),
            Link::Embed(url) => write!(f, "{}", url),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlatformLink {
    InstagramReel(String),
    InstagramPost(String),
    InstagramProfile(String),
    YoutubeVideo {
        video_id: String,
        timestamp: Option<u32>,
    },
    RedditShareLink {
        subreddit: String,
        share_id: String,
    },
    RedditPost {
        subreddit: String,
        post_id: String,
        comment_id: Option<String>,
    },
    Tweet {
        username: String,
        status_id: u64,
    },
}

impl PlatformLink {
    pub async fn alternative_links(self) -> Vec<Link> {
        match self {
            PlatformLink::InstagramReel(reel_id) => vec![
                Link::Embed(format!("https://www.ddinstagram.com/reel/{reel_id}/")),
                Link::Simple(format!("https://www.instagram.com/reel/{reel_id}/")),
            ],
            PlatformLink::InstagramPost(post_id) => vec![
                Link::Simple(format!("https://www.ddinstagram.com/p/{post_id}/")),
                Link::Simple(format!("https://www.instagram.com/p/{post_id}/")),
            ],
            PlatformLink::InstagramProfile(username) => vec![Link::Simple(format!(
                "https://www.instagram.com/{username}/"
            ))],
            PlatformLink::YoutubeVideo {
                video_id,
                timestamp,
            } => {
                if let Some(timestamp) = timestamp {
                    vec![Link::Simple(format!(
                        "https://youtu.be/{video_id}/?t={timestamp}"
                    ))]
                } else {
                    vec![Link::Simple(format!("https://youtu.be/{video_id}/"))]
                }
            }
            PlatformLink::RedditShareLink {
                subreddit,
                share_id,
            } => {
                if let Some(PlatformLink::RedditPost {
                    subreddit,
                    post_id,
                    comment_id,
                }) = resolve_reddit_share_link(&subreddit, &share_id).await
                {
                    alternative_reddit_links(&subreddit, &post_id, comment_id.as_deref())
                } else {
                    vec![]
                }
            }
            PlatformLink::RedditPost {
                subreddit,
                post_id,
                comment_id,
            } => alternative_reddit_links(&subreddit, &post_id, comment_id.as_deref()),
            PlatformLink::Tweet {
                username,
                status_id,
            } => vec![
                Link::Embed(format!(
                    "https://fxtwitter.com/{username}/status/{status_id}"
                )),
                Link::Simple(format!("https://x.com/{username}/status/{status_id}")),
            ],
        }
    }
}

pub fn find_platform_links(message: &str) -> Vec<PlatformLink> {
    LinkFinder::new()
        .kinds(&[LinkKind::Url])
        .links(message)
        .map(|link| link.as_str())
        .filter_map(|link| Url::parse(link).ok())
        .filter_map(get_platform_link)
        .collect()
}

pub fn get_platform_link(url: Url) -> Option<PlatformLink> {
    if url.scheme() != "https" && url.scheme() != "http" {
        return None;
    }

    match url.domain() {
        Some("instagram.com") | Some("www.instagram.com") => {
            match url
                .path_segments()
                .map(|it| it.filter(|s| !s.is_empty()))
                .map(|mut it| [it.next(), it.next(), it.next()])
                .unwrap_or([None; 3])
            {
                [Some("reel"), Some(reel_id), None] => {
                    Some(PlatformLink::InstagramReel(reel_id.to_string()))
                }
                [Some("p"), Some(post_id), None] => {
                    Some(PlatformLink::InstagramPost(post_id.to_string()))
                }
                [Some(profile_id), None, _] => {
                    Some(PlatformLink::InstagramProfile(profile_id.to_string()))
                }
                _ => None,
            }
        }
        Some("youtube.com") | Some("www.youtube.com") if url.path() == "/watch" => {
            let mut video_id = None;
            let mut timestamp = None;
            for (key, value) in url.query_pairs() {
                match key {
                    Cow::Borrowed("v") => video_id = Some(value.to_string()),
                    Cow::Borrowed("t") => timestamp = value.parse().ok(),
                    _ => {}
                }
            }

            video_id.map(|video_id| PlatformLink::YoutubeVideo {
                video_id,
                timestamp,
            })
        }
        Some("youtu.be") => {
            if let [Some(video_id), None] = url
                .path_segments()
                .map(|it| it.filter(|s| !s.is_empty()))
                .map(|mut it| [it.next(), it.next()])
                .unwrap_or([None; 2])
            {
                let mut timestamp = None;
                for (key, value) in url.query_pairs() {
                    if let Cow::Borrowed("t") = key {
                        timestamp = value.parse().ok()
                    }
                }

                Some(PlatformLink::YoutubeVideo {
                    video_id: video_id.to_string(),
                    timestamp,
                })
            } else {
                None
            }
        }
        Some("reddit.com") | Some("www.reddit.com") => {
            match url
                .path_segments()
                .map(|it| it.filter(|s| !s.is_empty()))
                .map(|mut it| [None; 7].map(|_: Option<&str>| it.next()))
                .unwrap_or([None; 7])
            {
                // /r/<subreddit>/s/<share_id>
                [Some("r"), Some(subreddit), Some("s"), Some(share_id), None, ..] => {
                    Some(PlatformLink::RedditShareLink {
                        subreddit: subreddit.to_string(),
                        share_id: share_id.to_string(),
                    })
                }
                // /r/<subreddit>/comments/<post_id>/comment/<comment_id>
                [Some("r"), Some(subreddit), Some("comments"), Some(post_id), Some("comment"), Some(comment_id), None] => {
                    Some(PlatformLink::RedditPost {
                        subreddit: subreddit.to_string(),
                        post_id: post_id.to_string(),
                        comment_id: Some(comment_id.to_string()),
                    })
                }
                // /r/<subreddit>/comments/<post_id>/<perhaps post name>
                [Some("r"), Some(subreddit), Some("comments"), Some(post_id), Some(_), None, ..]
                | [Some("r"), Some(subreddit), Some("comments"), Some(post_id), None, ..] => {
                    Some(PlatformLink::RedditPost {
                        subreddit: subreddit.to_string(),
                        post_id: post_id.to_string(),
                        comment_id: None,
                    })
                }
                _ => None,
            }
        }
        Some("twitter.com") | Some("www.twitter.com") | Some("x.com") | Some("www.x.com") => {
            if let [Some(username), Some("status"), Some(status_id), None] = url
                .path_segments()
                .map(|it| it.filter(|s| !s.is_empty()))
                .map(|mut it| [None; 4].map(|_: Option<&str>| it.next()))
                .unwrap_or([None; 4])
            {
                Some(PlatformLink::Tweet {
                    username: username.to_string(),
                    status_id: status_id.parse().ok()?,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{find_platform_links, PlatformLink};

    #[test]
    fn test_find_platform_links() {
        let message = "
            Lorem ipsum dolor sit amet, consectetur adipiscing elit.
            https://www.instagram.com/reel/AAAAAAAAAAA/?igsh=ZZZZZZZZZZZZZZZZ
            https://www.instagram.com/reel/BBBBBBBBBBB/
            https://www.instagram.com/reel/CCCCCCCCCCC?igsh=ZZZZZZZZZZZZZZZZ
            https://instagram.com/reel/DDDDDDDDDDD/?igsh=ZZZZZZZZZZZZZZZZ
            http://www.instagram.com/reel/EEEEEEEEEEE/?igsh=ZZZZZZZZZZZZZZZZ

            Sed laoreet sed arcu eget posuere.
            https://www.instagram.com/p/AAAAAAAAAAA/?igsh=ZZZZZZZZZZZZZZZZ
            https://www.instagram.com/p/BBBBBBBBBBB/
            https://www.instagram.com/p/CCCCCCCCCCC?igsh=ZZZZZZZZZZZZZZZZ
            https://instagram.com/p/DDDDDDDDDDD/?igsh=ZZZZZZZZZZZZZZZZ
            http://www.instagram.com/p/EEEEEEEEEEE/?igsh=ZZZZZZZZZZZZZZZZ

            Curabitur faucibus sodales metus a placerat.
            https://www.instagram.com/lorem_ipsum

            Etiam ac nisl non quam aliquet ultrices eu consectetur magna.
            https://youtube.com/channel?v=ABCD
            https://www.youtube.com/channel?v=ABCD
            https://www.youtube.com/watch?v=AAAAA_AA-AA&feature=featured
            https://www.youtube.com/watch?v=BBBBBBBBBBB&t=1234
            http://youtube.com/watch?v=CCCCCCCCCCC
            https://youtu.be/DDDDDDDDDDD?si=ZZZZZZZZZZZZZZZZ
            http://youtu.be/EEEEEEEEEEE?t=4321

            Pellentesque neque quam, vulputate id ornare quis, lobortis id lacus.
            https://www.reddit.com/r/subreddit/comments/AAAAAAA/some_post_name/?share_id=ZZZZZZZZZZZZZZZZZZZZZ&utm_content=1&utm_medium=ios_app&utm_name=ioscss&utm_source=share&utm_term=1
            https://www.reddit.com/r/subreddit/comments/AAAAAAA/comment/BBBBBBB/

            Morbi varius augue quis sem efficitur posuere.
            https://x.com/johndoe/status/123456789123456
            https://twitter.com/janedoe/status/988644234135645
            ";

        let links = find_platform_links(message);

        assert_eq!(
            vec![
                PlatformLink::InstagramReel("AAAAAAAAAAA".to_string()),
                PlatformLink::InstagramReel("BBBBBBBBBBB".to_string()),
                PlatformLink::InstagramReel("CCCCCCCCCCC".to_string()),
                PlatformLink::InstagramReel("DDDDDDDDDDD".to_string()),
                PlatformLink::InstagramReel("EEEEEEEEEEE".to_string()),
                PlatformLink::InstagramPost("AAAAAAAAAAA".to_string()),
                PlatformLink::InstagramPost("BBBBBBBBBBB".to_string()),
                PlatformLink::InstagramPost("CCCCCCCCCCC".to_string()),
                PlatformLink::InstagramPost("DDDDDDDDDDD".to_string()),
                PlatformLink::InstagramPost("EEEEEEEEEEE".to_string()),
                PlatformLink::InstagramProfile("lorem_ipsum".to_string()),
                PlatformLink::YoutubeVideo {
                    video_id: "AAAAA_AA-AA".to_string(),
                    timestamp: None,
                },
                PlatformLink::YoutubeVideo {
                    video_id: "BBBBBBBBBBB".to_string(),
                    timestamp: Some(1234),
                },
                PlatformLink::YoutubeVideo {
                    video_id: "CCCCCCCCCCC".to_string(),
                    timestamp: None,
                },
                PlatformLink::YoutubeVideo {
                    video_id: "DDDDDDDDDDD".to_string(),
                    timestamp: None,
                },
                PlatformLink::YoutubeVideo {
                    video_id: "EEEEEEEEEEE".to_string(),
                    timestamp: Some(4321),
                },
                PlatformLink::RedditPost {
                    subreddit: "subreddit".to_string(),
                    post_id: "AAAAAAA".to_string(),
                    comment_id: None,
                },
                PlatformLink::RedditPost {
                    subreddit: "subreddit".to_string(),
                    post_id: "AAAAAAA".to_string(),
                    comment_id: Some("BBBBBBB".to_string()),
                },
                PlatformLink::Tweet {
                    username: "johndoe".to_string(),
                    status_id: 123456789123456,
                },
                PlatformLink::Tweet {
                    username: "janedoe".to_string(),
                    status_id: 988644234135645,
                },
            ],
            links
        )
    }
}
