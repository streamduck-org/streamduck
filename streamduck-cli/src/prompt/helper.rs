use std::collections::HashMap;
use std::str::Split;
use itertools::Itertools;
use rustyline::completion::Completer;
use rustyline::{Context, Helper};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

#[derive(Default)]
pub struct CommandMap(pub HashMap<&'static str, CommandMap>);

fn get_command_map() -> CommandMap {
    let mut root = HashMap::new();

    root.insert("exit", Default::default());
    root.insert("help", Default::default());
    root.insert("select", Default::default());

    root.insert("device", {
        let mut commands = HashMap::new();

        commands.insert("list", Default::default());
        commands.insert("add", Default::default());
        commands.insert("remove", Default::default());

        CommandMap(commands)
    });

    root.insert("config", {
        let mut commands = HashMap::new();

        commands.insert("reload", {
            let mut commands = HashMap::new();

            commands.insert("all", Default::default());

            CommandMap(commands)
        });

        commands.insert("save", {
            let mut commands = HashMap::new();

            commands.insert("all", Default::default());

            CommandMap(commands)
        });

        commands.insert("import", Default::default());
        commands.insert("export", Default::default());

        CommandMap(commands)
    });

    root.insert("module", {
        let mut commands = HashMap::new();

        commands.insert("list", Default::default());
        commands.insert("info", Default::default());
        commands.insert("params", {
            let mut commands = HashMap::new();

            commands.insert("add", Default::default());
            commands.insert("remove", Default::default());
            commands.insert("set", Default::default());
            commands.insert("upload", Default::default());
            commands.insert("list", Default::default());

            CommandMap(commands)
        });

        CommandMap(commands)
    });

    root.insert("font", {
        let mut commands = HashMap::new();

        commands.insert("list", Default::default());

        CommandMap(commands)
    });

    root.insert("component", {
        let mut commands = HashMap::new();

        commands.insert("list", Default::default());
        commands.insert("info", Default::default());

        CommandMap(commands)
    });

    root.insert("image", {
        let mut commands = HashMap::new();

        commands.insert("list", Default::default());
        commands.insert("add", Default::default());
        commands.insert("remove", Default::default());

        CommandMap(commands)
    });

    root.insert("button", {
        let mut commands = HashMap::new();

        commands.insert("list", Default::default());
        commands.insert("info", Default::default());
        commands.insert("new", Default::default());
        commands.insert("from", Default::default());
        commands.insert("copy", Default::default());
        commands.insert("paste", Default::default());
        commands.insert("remove", Default::default());

        commands.insert("component", {
            let mut commands = HashMap::new();

            commands.insert("add", Default::default());
            commands.insert("remove", Default::default());
            commands.insert("params", {
                let mut commands = HashMap::new();

                commands.insert("add", Default::default());
                commands.insert("remove", Default::default());
                commands.insert("set", Default::default());
                commands.insert("upload", Default::default());
                commands.insert("list", Default::default());

                CommandMap(commands)
            });

            CommandMap(commands)
        });

        CommandMap(commands)
    });

    root.insert("brightness", Default::default());
    root.insert("back", Default::default());
    root.insert("press", Default::default());
    root.insert("stack", Default::default());

    CommandMap(root)
}

fn find_closest_command(mut line: Split<&str>, current_map: &CommandMap) -> Option<String> {
    if let Some(piece) = line.next() {
        if !piece.is_empty() {
            if let Some(found) = current_map.0.get(piece) {
                find_closest_command(line, found)
            } else {
                if line.next().is_none() {
                    for key in current_map.0.keys().sorted() {
                        if key.starts_with(piece) {
                            return Some(key[(piece.len())..].to_string());
                        }
                    }
                }

                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn get_candidates(mut line: Split<&str>, current_map: &CommandMap, current_pos: usize) -> Option<(usize, Vec<String>)> {
    if let Some(piece) = line.next() {
        if !piece.is_empty() {
            if let Some(found) = current_map.0.get(piece) {
                return get_candidates(line, found, current_pos + 1 + piece.len());
            } else {
                let mut candidates = vec![];

                if line.next().is_none() {
                    for key in current_map.0.keys().sorted() {
                        if key.starts_with(piece) {
                            candidates.push(key.to_string());
                        }
                    }
                }

                return Some((current_pos, candidates));
            }
        } else {
            Some((current_pos, current_map.0.keys().sorted().map(|x| x.to_string()).collect()))
        }
    } else {
        None
    }

}


pub struct StreamduckHelper;
impl Helper for StreamduckHelper {}
impl Validator for StreamduckHelper {}
impl Highlighter for StreamduckHelper {}

impl Completer for StreamduckHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let map = get_command_map();
        if let Some(result) = get_candidates(line.split(" "), &map, 0) {
            Ok(result)
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for StreamduckHelper {
    type Hint = String;

    fn hint(&self, line: &str, _: usize, _: &Context<'_>) -> Option<Self::Hint> {
        let map = get_command_map();
        find_closest_command(line.split(" "), &map)
    }
}