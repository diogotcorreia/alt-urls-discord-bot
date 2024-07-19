use std::fmt::Display;

use regex::Regex;
use regex_static::once_cell::sync::Lazy;

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
pub enum PlatformLink<'a> {
    InstagramReel(&'a str),
    InstagramPost(&'a str),
    InstagramProfile(&'a str),
    YoutubeVideo(&'a str),
}

impl<'a> PlatformLink<'a> {
    pub fn alternative_links(&self) -> Vec<Link> {
        match self {
            PlatformLink::InstagramReel(reel_id) => vec![
                Link::Embed(format!("https://www.ddinstagram.com/reel/{reel_id}/")),
                Link::Simple(format!("https://www.instagram.com/reel/{reel_id}/")),
            ],
            PlatformLink::InstagramPost(post_id) => vec![Link::Simple(format!(
                "https://www.instagram.com/p/{post_id}/"
            ))],
            PlatformLink::InstagramProfile(username) => vec![Link::Simple(format!(
                "https://www.instagram.com/{username}/"
            ))],
            PlatformLink::YoutubeVideo(video_id) => {
                vec![Link::Simple(format!("https://youtu.be/{video_id}/"))]
            }
        }
    }
}

pub fn find_platform_links(message: &str) -> Vec<PlatformLink<'_>> {
    macro_rules! handle_pattern {
        ($pattern: expr, $fn: expr) => {
            $pattern
                .captures_iter(message)
                .map(|c| c.extract())
                .for_each($fn)
        };
    }
    let mut links_found = vec![];

    handle_pattern!(INSTAGRAM_REEL_PATTERN, |(_, [reel_id])| links_found
        .push(PlatformLink::InstagramReel(reel_id)));
    handle_pattern!(INSTAGRAM_POST_PATTERN, |(_, [post_id])| links_found
        .push(PlatformLink::InstagramPost(post_id)));
    handle_pattern!(INSTAGRAM_PROFILE_PATTERN, |(_, [profile_id])| links_found
        .push(PlatformLink::InstagramProfile(profile_id)));

    handle_pattern!(YOUTUBE_VIDEO_PATTERN, |(_, [video_id])| links_found
        .push(PlatformLink::YoutubeVideo(video_id)));

    links_found
}

static INSTAGRAM_REEL_PATTERN: Lazy<Regex> =
    regex_static::lazy_regex!(r"https?://(?:www\.)?instagram\.com/reel/(\w{1,20})/?(?:\?|\b)");
static INSTAGRAM_POST_PATTERN: Lazy<Regex> =
    regex_static::lazy_regex!(r"https?://(?:www\.)?instagram\.com/p/(\w{1,20})/?(?:\?|\b)");
// https://stackoverflow.com/questions/32543090/instagram-username-regex-php
static INSTAGRAM_PROFILE_PATTERN: Lazy<Regex> =
    regex_static::lazy_regex!(r"https?://(?:www\.)?instagram\.com/(\w{1,30})/?(?:\?|\s|$)");
static YOUTUBE_VIDEO_PATTERN: Lazy<Regex> = regex_static::lazy_regex!(
    r"https?://youtu\.be/([\w-]{1,16})|https?://(?:www\.)?youtube\.com/watch\?v=([\w-]{1,16})"
);

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
            https://www.youtube.com/watch?v=AAAAA_AA-AA&feature=featured
            https://www.youtube.com/watch?v=BBBBBBBBBBB
            http://youtube.com/watch?v=CCCCCCCCCCC
            https://youtu.be/DDDDDDDDDDD?si=ZZZZZZZZZZZZZZZZ
            http://youtu.be/EEEEEEEEEEE
            ";

        let links = find_platform_links(message);

        assert_eq!(
            vec![
                PlatformLink::InstagramReel("AAAAAAAAAAA"),
                PlatformLink::InstagramReel("BBBBBBBBBBB"),
                PlatformLink::InstagramReel("CCCCCCCCCCC"),
                PlatformLink::InstagramReel("DDDDDDDDDDD"),
                PlatformLink::InstagramReel("EEEEEEEEEEE"),
                PlatformLink::InstagramPost("AAAAAAAAAAA"),
                PlatformLink::InstagramPost("BBBBBBBBBBB"),
                PlatformLink::InstagramPost("CCCCCCCCCCC"),
                PlatformLink::InstagramPost("DDDDDDDDDDD"),
                PlatformLink::InstagramPost("EEEEEEEEEEE"),
                PlatformLink::InstagramProfile("lorem_ipsum"),
                PlatformLink::YoutubeVideo("AAAAA_AA-AA"),
                PlatformLink::YoutubeVideo("BBBBBBBBBBB"),
                PlatformLink::YoutubeVideo("CCCCCCCCCCC"),
                PlatformLink::YoutubeVideo("DDDDDDDDDDD"),
                PlatformLink::YoutubeVideo("EEEEEEEEEEE"),
            ],
            links
        )
    }
}
