#![warn(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
//    clippy::pedantic,
    clippy::cargo,
)]
use std::cmp::Ordering;
use std::env;
use std::io;
use std::io::Write;
use std::process;
fn main() {
    let args = env::args().collect::<Vec<String>>();
    //dbg!(&args);
    if args[1] == "decode" {
        let mut stdout = io::stdout().lock();
        let Some(query) = detect_query(&args[2]) else {
            process::exit(1);
        };
        let string = format!(
            "{}\n{}\n{}",
            query.history_entry, query.starting_index, query.ending_index
        );
        let _ = stdout.write_all(string.as_bytes());
    } else if args[1] == "parse" {
        let a = args[2].parse::<isize>().unwrap();
        let b = args[3].parse::<isize>().unwrap();
        let mut stdout = io::stdout().lock();
        let arguments = get_arguments(&args[4]);
        //dbg!(a, b, arguments.len());
        let (a, b, arguments) = horrible_mess_that_seems_like_it_works(a, b, arguments);
        //dbg!(a, b, arguments.len());
        let _ = stdout.write_all(arguments[a..b].join("\n").as_bytes());
    }
}

// fish like indexing of vector/lists/arrays
fn horrible_mess_that_seems_like_it_works(
    mut a: isize,
    mut b: isize,
    mut arguments: Vec<String>,
) -> (usize, usize, Vec<String>) {
    if a < -(arguments.len() as isize) && b < -(arguments.len() as isize) {
        return (0, 0, arguments);
    }
    match (a.cmp(&0), b.cmp(&0), a.cmp(&b)) {
        (Ordering::Less, Ordering::Less, Ordering::Greater) => {
            //dbg!("this branch 5");
            a = -a - 2;
            b = -b;
            arguments.reverse();
        }
        (_, Ordering::Greater, Ordering::Greater) => {
            //dbg!("this branch 4");
            a = arguments.len() as isize - a;
            b = arguments.len() as isize - b + 1;
            arguments.reverse();
        }
        (Ordering::Less, Ordering::Less, _) => {
            //dbg!("this branch 3");
            a += arguments.len() as isize;
            b += arguments.len() as isize + 1;
        }
        (Ordering::Less, _, _) => {
            //dbg!("this branch 2");
            arguments.reverse();
            a = -a - 1;
            b = arguments.len() as isize - b + 1;
        }
        (_, Ordering::Less, _) => {
            //dbg!("this branch 1");
            a -= 1;
            b += arguments.len() as isize;
        }
        _ => a -= 1,
    }
    //dbg!(b);
    if a > b {
        return (0, 0, arguments);
    }
    if b > arguments.len() as isize {
        b = arguments.len() as isize;
    }
    //dbg!(b);
    if a < 0 {
        a = 0;
    }
    (a as usize, b as usize, arguments)
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
            (' ', _) => panic!("unreachable"),

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
            value.history_entry = history_entry.parse().expect("unreachable");
        }
        if !starting_index.is_empty() {
            if starting_index == "-" {
                starting_index = "-1".to_string();
            }
            value.starting_index = starting_index.parse().expect("unreachable");
        }
        if !ending_index.is_empty() {
            if ending_index == "-" {
                ending_index = "-1".to_string();
            }
            value.ending_index = ending_index.parse().expect("unreachable");
        }
        Some(value)
    } else {
        None
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum State {
    Escaped,
    InDoubleQuotes,
    InSingleQuotes,
    Normal,
}
const BACKSLASH: char = '\\';
const DOUBLE_QUOTE: char = '"';
const SINGLE_QOUTE: char = '\'';
const SPACE: char = ' ';

fn get_arguments(string: &str) -> Vec<String> {
    let mut arguments: Vec<String> = vec![];
    let mut current_argument = String::new();
    let mut previous_character = 0b0 as char; // null char
    let mut state = State::Normal;
    for character in string.chars() {
        #[rustfmt::skip]
        match (previous_character, character, &state) { // pedantic gave some weird suggestions that changed how the code behave please ignore it
            (BACKSLASH,       _,        State::Normal        ) => state = State::Escaped,
            (DOUBLE_QUOTE,    _,        State::Normal        ) => state = State::InDoubleQuotes,
            (DOUBLE_QUOTE,    _,        State::InDoubleQuotes) => state = State::Normal, // Go back to normal on quote end
            ( _,              _,        State::InDoubleQuotes) => state = State::InDoubleQuotes, // Keep quote going
            (SINGLE_QOUTE,    _,        State::Normal        ) => state = State::InSingleQuotes,
            (SINGLE_QOUTE,    _,        State::InSingleQuotes) => state = State::Normal,
            ( _,              _,        State::InSingleQuotes) => state = State::InSingleQuotes,
            _ => state = State::Normal,
        };
        //dbg!(character, previous_character, &current_argument, &arguments, &state);
        match (previous_character, character, &state, &current_argument) {
            (_, DOUBLE_QUOTE, State::Normal, _) if !current_argument.is_empty() => {
                //dbg!("branch 3");
                current_argument.push(character);
                arguments.push(current_argument.clone());
                current_argument.clear();
            }

            (_, SPACE, State::Normal, _) if !current_argument.is_empty() => {
                //dbg!("branch 2");
                arguments.push(current_argument.clone());
                current_argument.clear();
            }

            (_, _, _, _) if character != SPACE || state != State::Normal => {
                //dbg!("branch 1");
                current_argument.push(character);
            }

            _ => (),
        }

        previous_character = character;
    }

    // When end is reached don't forget to add the last argument!
    if !current_argument.is_empty() {
        // and to check if such is not empty...
        arguments.push(current_argument.clone());
    }

    arguments
}

#[cfg(test)]
mod test {
    use super::*;
    use std::process::Command; //idk why clippy complains about this, it used in the tests
    fn tester_function(test_strings: &[&str], expected_result: &[Vec<&str>]) {
        assert!(test_strings.len() == expected_result.len());
        for string_index in 0..test_strings.len() {
            //dbg!(get_arguments(test_strings[string_index]));
            //dbg!(&expected_result[string_index]);
            assert!(get_arguments(test_strings[string_index]) == expected_result[string_index]);
        }
    }
    #[test]
    fn single_argument_test() {
        let mut test_strings: Vec<&str> = vec![];
        let mut expected_result: Vec<Vec<&str>> = vec![];

        test_strings.push("test");
        expected_result.push(Vec::from(["test"]));

        tester_function(&test_strings, &expected_result);
    }
    #[test]
    fn space_test() {
        let mut test_strings: Vec<&str> = vec![];
        let mut expected_result: Vec<Vec<&str>> = vec![];

        test_strings.push("test string");
        expected_result.push(Vec::from(["test", "string"]));

        test_strings.push("test string with more spaces");
        expected_result.push(Vec::from(["test", "string", "with", "more", "spaces"]));

        test_strings.push("test string     ");
        expected_result.push(Vec::from(["test", "string"]));

        test_strings.push("   test string");
        expected_result.push(Vec::from(["test", "string"]));

        tester_function(&test_strings, &expected_result);
    }
    #[test]
    fn escapped_space_test() {
        let mut test_strings: Vec<&str> = vec![];
        let mut expected_result: Vec<Vec<&str>> = vec![];

        test_strings.push("test\\ string");
        expected_result.push(Vec::from(["test\\ string"]));

        test_strings.push("\\test string");
        expected_result.push(Vec::from(["\\test", "string"]));

        test_strings.push("\\ test string");
        expected_result.push(Vec::from(["\\ test", "string"]));

        test_strings.push("test string\\");
        expected_result.push(Vec::from(["test", "string\\"]));

        test_strings.push("test string\\ ");
        expected_result.push(Vec::from(["test", "string\\ "]));

        tester_function(&test_strings, &expected_result);
    }

    #[test]
    fn double_qoutes_test() {
        let mut test_strings: Vec<&str> = vec![];
        let mut expected_result: Vec<Vec<&str>> = vec![];

        test_strings.push("\"test\"");
        expected_result.push(Vec::from(["\"test\""]));

        test_strings.push("\"test string\"");
        expected_result.push(Vec::from(["\"test string\""]));

        test_strings.push("\"test\" string\"");
        expected_result.push(Vec::from(["\"test\"", "string\""]));

        test_strings.push("\"test string");
        expected_result.push(Vec::from(["\"test string"]));

        tester_function(&test_strings, &expected_result);
    }

    #[test]
    fn single_qoutes_test() {
        let mut test_strings: Vec<&str> = vec![];
        let mut expected_result: Vec<Vec<&str>> = vec![];

        test_strings.push("\'test\'");
        expected_result.push(Vec::from(["\'test\'"]));

        test_strings.push("\'test string\'");
        expected_result.push(Vec::from(["\'test string\'"]));

        test_strings.push("\'test\' string\'");
        expected_result.push(Vec::from(["\'test\'", "string\'"]));

        test_strings.push("\'test string");
        expected_result.push(Vec::from(["\'test string"]));

        tester_function(&test_strings, &expected_result);
    }

    #[test]
    fn mixed_quotes_test() {
        let mut test_strings: Vec<&str> = vec![];
        let mut expected_result: Vec<Vec<&str>> = vec![];

        test_strings.push("\'test \"string\"\'");
        expected_result.push(Vec::from(["\'test \"string\"\'"]));

        test_strings.push("\"test \'string\'\"");
        expected_result.push(Vec::from(["\"test \'string\'\""]));

        test_strings.push("\'test\"string\"\'");
        expected_result.push(Vec::from(["\'test\"string\"\'"]));

        test_strings.push("\"test\'string\'\"");
        expected_result.push(Vec::from(["\"test\'string\'\""]));

        test_strings.push("\'test \"string\'\"");
        expected_result.push(Vec::from(["\'test \"string\'\""]));

        test_strings.push("\"test\'string\"\'");
        expected_result.push(Vec::from(["\"test\'string\"\'"]));

        test_strings.push("\'test\"string\'\"");
        expected_result.push(Vec::from(["\'test\"string\'\""]));

        test_strings.push("\"test\'string\"\'");
        expected_result.push(Vec::from(["\"test\'string\"\'"]));

        tester_function(&test_strings, &expected_result);
    }
    #[test]
    fn test33() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (1, 1);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        //dbg!(a, b, &list);
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        //dbg!(new_a, new_b, &new_list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test32() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (8, 8);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test31() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (8, -12);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test30() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-11, 5);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test29() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-8, 5);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test28() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-8, 11);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test27() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (8, 11);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test26() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (11, 5);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test25() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (8, 5);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test24() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (11, 12);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test23() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (12, 11);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test22() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (11, 11);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test21() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-11, -11);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test20() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-13, -12);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn te19() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-11, -12);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test18() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-11, -1);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test17() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-4, -4);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test16() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-7, -5);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test15() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-3, -1);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test14() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-1, -3);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test13() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (1, 11);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test12() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-3, 3);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test11() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (-2, 2);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
    #[test]
    fn test10() {
        let list = Vec::from(["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "0".to_string()]);

        let (a, b) = (1, 10);
        let index = format!("{a}..{b}");
        let command = format!("set list {}; echo $list[{index}]", list.join(" "));
        
        let raw_output = Command::new("fish")
            .arg("-c")
            .arg(command)
            .output()
            .expect("something went wrong");
        let expected_result = String::from_utf8_lossy(raw_output.stdout.as_slice());
        let (new_a, new_b, new_list) = horrible_mess_that_seems_like_it_works(a, b, list);
        let mut result = new_list[new_a..new_b].join(" ");
        result.push('\n');

        debug_assert_eq!(&expected_result, &result);
    }
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
            ending_index: -1,
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
            ending_index: -1,
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
    fn history_and_starting_index_test() {
        let string = "!3:4";
        let expected_result = Some(Query {
            history_entry: 3,
            starting_index: 4,
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
