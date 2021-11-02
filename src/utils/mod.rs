use linkify::LinkFinder;

pub mod google;

/**
 * extract the first url of a string
 */
pub fn extract_url(input: &str) -> Option<&str> {
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(input).collect();
    links.get(0).map(|l| l.as_str())
}
