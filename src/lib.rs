// take a body of text
// split it into pieces that are `max_tweet_length` or less
// splits are only valid on whitespace
// try to preserve whitespace
// if whitespace falls on a split, discard it
// if there is no whitespace, split on charlength
// trim trailing whitespace from text

use lazy_static::*;
use regex::Regex;

#[derive(Clone, Copy, Debug)]
struct Span<'a> {
    regex_match: regex::Match<'a>,
    length: usize,
}

impl<'a> Span<'a> {
    fn new(regex_match: regex::Match<'a>) -> Self {
        Self {
            regex_match,
            length: regex_match.end() - regex_match.start(),
        }
    }

    fn len(&self) -> usize {
        self.length
    }

    fn start_end(&self) -> (usize, usize) {
        (self.regex_match.start(), self.regex_match.end())
    }
}

#[derive(Clone, Debug)]
pub enum TweetSplitError {
    MaxTweetLengthTooShort { details: String },
}

impl std::fmt::Display for TweetSplitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TweetSplitError::MaxTweetLengthTooShort { details } => write!(f, "{}", details),
        }
    }
}

impl std::error::Error for TweetSplitError {
    fn description(&self) -> &str {
        match self {
            TweetSplitError::MaxTweetLengthTooShort { details } => details,
        }
    }
}

pub fn split_text(input: &str, max_tweet_length: usize) -> Result<Vec<String>, TweetSplitError> {
    let input = input.trim();

    let mut spaces = SPACE_MATCHER
        .find_iter(input)
        .map(Span::new)
        .collect::<Vec<Span>>();

    if !spaces.is_empty() {
        let words = WORD_MATCHER
            .find_iter(input)
            .map(Span::new)
            .collect::<Vec<Span>>();

        // if there are less spaces than words due to trimming,
        // add enough spaces so that `spaces.len() == words.len()`
        // this is safe because `spaces` in this branch
        // must have len > 0
        loop {
            if spaces.len() < words.len() {
                spaces.push(spaces[spaces.len() - 1].clone())
            } else {
                break;
            }
        }

        let words_spaces = words.into_iter().zip(spaces);

        let mut span_groups: Vec<Vec<Span>> = vec![];

        let mut current_tweet_length = 0usize;

        let mut current_span_group: Vec<Span> = vec![];

        for (word, space) in words_spaces {
            let word_length = word.len();

            if word_length + current_tweet_length <= max_tweet_length {
                current_span_group.push(word);
                current_tweet_length += word_length;

                let space_length = space.len();
                if space_length + current_tweet_length <= max_tweet_length {
                    current_span_group.push(space);
                    current_tweet_length += space_length;
                }
            } else if word_length <= max_tweet_length {
                current_tweet_length = 0;
                span_groups.push(current_span_group);
                current_span_group = vec![];

                current_span_group.push(word);
                current_tweet_length += word_length;

                let space_length = space.len();

                if space_length + current_tweet_length < max_tweet_length {
                    current_span_group.push(space);
                    current_tweet_length += space_length;
                }
            } else {
                return Err(TweetSplitError::MaxTweetLengthTooShort {
                    details: format!(
                        "Tweet length of {} is too short to split only on whitespace.",
                        max_tweet_length
                    ),
                });
            }
        }

        // add the final span group
        span_groups.push(current_span_group);

        Ok(span_groups
            .iter()
            .map(|span_group| {
                let tweet = span_group
                    .iter()
                    .map(|span| {
                        let (start, end) = span.start_end();
                        &input[start..end]
                    })
                    .collect::<Vec<&str>>()
                    .join("");

                tweet.trim_end().to_string()
            })
            .collect::<Vec<String>>())
    } else {
        Err(TweetSplitError::MaxTweetLengthTooShort {
            details: format!(
                "Tweet length of {} is too short to split only on whitespace.",
                max_tweet_length
            ),
        })
    }
}

lazy_static! {
    static ref WORD_MATCHER: Regex = Regex::new(r"\S+").unwrap();
    static ref SPACE_MATCHER: Regex = Regex::new(r"\s+").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRAITOROUS_EIGHT: &str = "The traitorous eight was a group of eight employees who left Shockley Semiconductor Laboratory in 1957 to found Fairchild Semiconductor. William Shockley had in 1956 recruited a group of young PhD graduates with the goal to develop and produce new semiconductor devices. While Shockley had received a Nobel Prize in Physics and was an experienced researcher and teacher, his management of the group was authoritarian and unpopular. This was accentuated by Shockley's research focus not proving fruitful. After the demand for Shockley to be replaced was rebuffed, the eight left to form their own company.";

    #[test]
    fn it_splits() {
        let input = "aaaaaaaaa bbbbbbbbb ccccccccc ddddddddd eeeeeeeee ";

        let splits = split_text(&input, 10).unwrap();

        assert_eq!(splits.len(), 5);
    }

    #[test]
    fn it_trims_spaces_at_splits() {
        let input = "aaaaaaaaa bbbbbbbbb ccccccccc ddddddddd eeeeeeeee ";

        let splits = split_text(&input, 10).unwrap();

        for split in splits {
            assert_eq!(split.len(), 9);
        }
    }

    #[test]
    fn it_properly_splits_at_smaller_char_sizes() {
        let input = TRAITOROUS_EIGHT;

        let splits = split_text(input, 25).unwrap();

        assert_eq!(splits[0], "The traitorous eight was");
    }

    #[test]
    fn it_trims_trailing_spaces() {
        let input = TRAITOROUS_EIGHT;

        for max_tweet_length in 14..=250 {
            let splits = split_text(input, max_tweet_length).unwrap();

            for split in splits {
                assert_ne!(split.chars().collect::<Vec<char>>().last().unwrap(), &' ');
            }
        }
    }
}
