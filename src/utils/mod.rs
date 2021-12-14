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

#[cfg(test)]
mod tests {
    #[test]
    fn extract_url() {
        assert_eq!(
            super::extract_url("abc http://www.google.com def").unwrap(),
            "http://www.google.com"
        );
    }

    #[test]
    fn extract_first_url() {
        assert_eq!(
            super::extract_url("abc http://www.google.com def http://www.twitter.com ghi").unwrap(),
            "http://www.google.com"
        );
    }
}
