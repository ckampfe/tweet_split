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
enum Span<'a> {
    Word {
        regex_match: regex::Match<'a>,
        length: usize,
    },
    Space {
        regex_match: regex::Match<'a>,
        length: usize,
    },
}

trait Lengthable {
    fn len(&self) -> usize;
    fn start_end(&self) -> (usize, usize);
}

impl Lengthable for Span<'_> {
    fn len(&self) -> usize {
        match self {
            Span::Word { length, .. } => *length,
            Span::Space { length, .. } => *length,
        }
    }

    fn start_end(&self) -> (usize, usize) {
        match self {
            Span::Word { regex_match, .. } => (regex_match.start(), regex_match.end()),
            Span::Space { regex_match, .. } => (regex_match.start(), regex_match.end()),
        }
    }
}

pub fn split_text(input: &str, max_tweet_length: usize) -> Vec<String> {
    let input = input.trim_start();

    let spaces = SPACE_MATCHER
        .find_iter(input)
        .map(|space_match| Span::Space {
            regex_match: space_match,
            length: space_match.end() - space_match.start(),
        })
        .collect::<Vec<Span>>();

    let has_spaces = spaces.len() > 0;

    if has_spaces {
        let words = WORD_MATCHER
            .find_iter(input)
            .map(|word_match| Span::Word {
                regex_match: word_match,
                length: word_match.end() - word_match.start(),
            })
            .collect::<Vec<Span>>();

        let words_spaces = words.into_iter().zip(spaces);

        let mut span_groups: Vec<Vec<Span>> = vec![];

        let mut current_tweet_length = 0usize;

        let mut current_span_group: Vec<Span> = vec![];

        for (word, space) in words_spaces {
            // println!("{}", current_tweet_length);
            // println!("{:?}", word);
            let word_length = word.len();

            if word_length + current_tweet_length <= max_tweet_length {
                current_span_group.push(word);
                current_tweet_length += word_length;

                let space_length = space.len();
                if space_length + current_tweet_length < max_tweet_length {
                    current_span_group.push(space);
                    current_tweet_length += space_length;
                }
            } else {
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
            }
        }

        // add the final span group
        span_groups.push(current_span_group);

        span_groups
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
            .collect::<Vec<String>>()
    } else {
        input
            .chars()
            .collect::<Vec<char>>()
            .chunks(max_tweet_length)
            .into_iter()
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
    }
}

lazy_static! {
    static ref WORD_MATCHER: Regex = Regex::new(r"\S+").unwrap();
    static ref SPACE_MATCHER: Regex = Regex::new(r"\s+").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_splits() {
        let input = "aaaaaaaaa bbbbbbbbb ccccccccc ddddddddd eeeeeeeee ";

        let splits = split_text(&input, 10);

        assert_eq!(splits.len(), 5);
    }

    #[test]
    fn it_trims_spaces_at_splits() {
        let input = "aaaaaaaaa bbbbbbbbb ccccccccc ddddddddd eeeeeeeee ";

        let splits = split_text(&input, 10);

        for split in splits {
            assert_eq!(split.len(), 9);
        }
    }

    #[test]
    fn in_the_absence_of_spaces_it_splits_words() {
        let input = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeee";

        let splits = split_text(&input, 10);

        for split in splits {
            println!("{:?}", split);
            assert_eq!(split.len(), 10);
        }
    }
}
