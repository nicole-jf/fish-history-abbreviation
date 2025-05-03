#![warn(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
//    clippy::pedantic,
    clippy::cargo,
)]
use std::env;
use std::io;
use std::io::Write;
use std::process;
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mut stdout = io::stdout().lock();
    let Some(query) = detect_query(&args[1]) else {
        process::exit(1);
    };
    let tmp = &[
        query.history_entry.to_string(),
        query.starting_index.to_string(),
        query.ending_index.to_string(),
    ];
    let string = tmp.join("\n");
    let _ = stdout.write_all(string.as_bytes());
}
#[derive(Debug, PartialEq, Clone, Copy)]
struct Query {
    history_entry: isize,
    starting_index: isize,
    ending_index: isize,
}

enum QuerySyntax {
    NotStarted,
    Prefix,
    HistoryEntryShorthand,
    HistoryEntry,
    Separator,
    StartingIndex,
    Separator2,
    EndingIndex,
}

// ASSUMPTIONS: THERE SHALL BE NO SPACES IN THE STRING PARSED BY `detect_query()`
fn detect_query(string: &str) -> Option<Query> {
    let mut state: QuerySyntax = QuerySyntax::NotStarted;
    let mut history_entry: String = String::new();
    let mut starting_index: String = String::new();
    let mut ending_index: String = String::new();

    for character in string.chars() {
        #[rustfmt::skip]
        match (character, state) {
        //    (' ', _) => panic!("unreachable"),

            ('!', QuerySyntax::NotStarted) => state = QuerySyntax::Prefix,
            ('!', QuerySyntax::Prefix) => {
                state = QuerySyntax::HistoryEntryShorthand;
                history_entry.push('1');
            },
            ('0'..='9', QuerySyntax::Prefix |
                        QuerySyntax::HistoryEntry) |
            ('-', QuerySyntax::Prefix) => {
                state = QuerySyntax::HistoryEntry;
                history_entry.push(character);
            },

            (':', QuerySyntax::HistoryEntry |
                  QuerySyntax::HistoryEntryShorthand |
                  QuerySyntax::Prefix) => {
                state = QuerySyntax::Separator;
            },
            ('0'..='9', QuerySyntax::Separator |
                        QuerySyntax::StartingIndex) |
            ('-', QuerySyntax::Separator) => {
                state = QuerySyntax::StartingIndex;
                starting_index.push(character);
            }

            ('.', QuerySyntax::StartingIndex |
                  QuerySyntax::Separator |
                  QuerySyntax::Separator2) => {
                state = QuerySyntax::Separator2;
            },
            ('0'..='9', QuerySyntax::Separator2 |
                        QuerySyntax::EndingIndex) |
            ('-', QuerySyntax::Separator2) => {
                state = QuerySyntax::EndingIndex;
                ending_index.push(character);
            }
            _ => state = QuerySyntax::NotStarted,
        };
    }

    if !history_entry.is_empty() || !starting_index.is_empty() {
        let mut value = Query {
            history_entry: 1,
            starting_index: 1,
            ending_index: -1,
        };
        if !history_entry.is_empty() {
            if history_entry == "-" {
                history_entry = "-1".to_string();
            }
            value.history_entry = history_entry.parse().unwrap_or_else(|_| process::exit(1));
        }
        if !starting_index.is_empty() {
            if starting_index == "-" {
                starting_index = "-1".to_string();
            }
            value.starting_index = starting_index.parse().unwrap_or_else(|_| process::exit(1));
        }
        if !ending_index.is_empty() {
            if ending_index == "-" {
                ending_index = "-1".to_string();
            }
            value.ending_index = ending_index.parse().unwrap_or_else(|_| process::exit(1));
        } else if !starting_index.is_empty() {
            value.ending_index = value.starting_index;
        }
        Some(value)
    } else {
        None
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test3() {
        let string = "!!:..-24";
        let expected_result = Some(Query {
            history_entry: 1,
            starting_index: 1,
            ending_index: -24,
        });
        let result = detect_query(string);
        //    //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => {
                //dbg!(expected_result, result);
                panic!("a and b not equal");
            }
        }
    }
    #[test]
    fn test2() {
        let string = "!!:-..-2";
        let expected_result = Some(Query {
            history_entry: 1,
            starting_index: -1,
            ending_index: -2,
        });
        let result = detect_query(string);
        //    //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => {
                //dbg!(expected_result, result);
                panic!("a and b not equal");
            }
        }
    }
    #[test]
    fn test1() {
        let string = "!!:1..-2";
        let expected_result = Some(Query {
            history_entry: 1,
            starting_index: 1,
            ending_index: -2,
        });
        let result = detect_query(string);
        //    //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => {
                //dbg!(expected_result, result);
                panic!("a and b not equal");
            }
        }
    }
    #[test]
    fn tmp_test() {
        let string = "!1:1.-2";
        let expected_result = Some(Query {
            history_entry: 1,
            starting_index: 1,
            ending_index: -2,
        });
        let result = detect_query(string);
        //    //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => {
                //dbg!(expected_result, result);
                panic!("a and b not equal");
            }
        }
    }
    #[test]
    fn negative_starting_index_only_test() {
        let string = "!:-23";
        let expected_result = Some(Query {
            history_entry: 1,
            starting_index: -23,
            ending_index: -23,
        });
        let result = detect_query(string);
        //    //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => {
                //dbg!(expected_result, result);
                panic!("a and b not equal");
            }
        }
    }
    #[test]
    fn history_and_starting_index_negative_test() {
        let string = "!-1:-23";
        let expected_result = Some(Query {
            history_entry: -1,
            starting_index: -23,
            ending_index: -23,
        });
        let result = detect_query(string);
        //    //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn history_only_negative_test() {
        let string = "!-1";
        let expected_result = Some(Query {
            history_entry: -1,
            starting_index: 1,
            ending_index: -1,
        });
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn history_and_starting_index_test2() {
        let string = "!42:69";
        let expected_result = Some(Query {
            history_entry: 42,
            starting_index: 69,
            ending_index: 69,
        });
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn history_and_starting_index_test() {
        let string = "!3:4";
        let expected_result = Some(Query {
            history_entry: 3,
            starting_index: 4,
            ending_index: 4,
        });
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn history_only_test() {
        let string = "!1";
        let expected_result = Some(Query {
            history_entry: 1,
            starting_index: 1,
            ending_index: -1,
        });
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn history_with_bigger_number_test() {
        let string = "!10";
        let expected_result = Some(Query {
            history_entry: 10,
            starting_index: 1,
            ending_index: -1,
        });
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn number_alone_test() {
        let string = "1";
        let expected_result: Option<Query> = None;
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn empty_test() {
        let string = "!";
        let expected_result: Option<Query> = None;
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn empty_test_with_junk() {
        let string = "!;";
        let expected_result: Option<Query> = None;
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn nothing_test() {
        let string = "";
        let expected_result: Option<Query> = None;
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
    #[test]
    fn gibberish_test() {
        let string = "sd!fdkfdkaajf!kf";
        let expected_result: Option<Query> = None;
        let result = detect_query(string);
        //dbg!(string, expected_result, result);
        match (&expected_result, &result) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => (),
            _ => panic!("a and b not equal"),
        }
    }
}
