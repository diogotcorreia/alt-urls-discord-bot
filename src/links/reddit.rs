// Specific logic for Reddit, since requests need to be made to get the clean URL

use reqwest::{header::LOCATION, redirect, Client};
use url::Url;

use super::{Link, PlatformLink};

pub async fn resolve_reddit_share_link(subreddit: &str, share_id: &str) -> Option<PlatformLink> {
    let client = Client::builder()
        .redirect(redirect::Policy::none())
        .user_agent("curl/8.7.1") // otherwise endpoint returns 403
        .build()
        .ok()?;

    let response = client
        .get(format!("https://www.reddit.com/r/{subreddit}/s/{share_id}"))
        .send()
        .await
        .ok()?;
    let real_link = response.headers().get(LOCATION)?;
    let real_link = Url::parse(real_link.to_str().ok()?).ok()?;
    dbg!(&real_link);

    // filter to avoid infinite recursion
    PlatformLink::try_from(real_link)
        .ok()
        .filter(|pl| matches!(pl, PlatformLink::RedditPost { .. }))
}

pub fn alternative_reddit_links(
    subreddit: &str,
    post_id: &str,
    comment_id: Option<&str>,
) -> Vec<Link> {
    if let Some(comment_id) = comment_id {
        vec![
            Link::Simple(format!(
                "https://www.reddit.com/r/{subreddit}/comments/{post_id}/comment/{comment_id}"
            )),
            Link::Simple(format!(
                "https://old.reddit.com/r/{subreddit}/comments/{post_id}/comment/{comment_id}"
            )),
        ]
    } else {
        vec![
            Link::Simple(format!(
                "https://www.reddit.com/r/{subreddit}/comments/{post_id}"
            )),
            Link::Simple(format!(
                "https://old.reddit.com/r/{subreddit}/comments/{post_id}"
            )),
        ]
    }
}
