use flexi_logger::{style, DeferredNow};
use log::Record;
use nu_ansi_term::Color;

pub fn shorten_module_path(input: &str, max_width: usize) -> String {
    let mut split = input.split("::");

    let Some(root) = split.next() else {
        return input.to_string();
    };

    let splits = split.collect::<Vec<&str>>();
    let split_count = splits.len();

    let mut remaining_width = max_width - root.len();
    let mut segments: Vec<&str> = vec![];

    for segment in splits.into_iter().rev() {
        if segment.len() > remaining_width {
            break
        }

        remaining_width -= segment.len();
        segments.insert(0, segment);
    }

    let val = if segments.len() <= 0 {
        root.to_string()
    } else {
        if split_count == segments.len() {
            format!("{}::{}", root, segments.join("::"))
        } else {
            format!("{}::_::{}", root, segments.join("::"))
        }
    };

    format!("{:>1$} ", val, max_width + 8)
}

pub fn log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();

    let module_path = record.module_path().unwrap_or("<unnamed>");

    write!(
        w,
        "{} {} [{}] ",
        now.format("%Y-%m-%d %H:%M:%S"),
        style(level).paint(format!("{:5}", level.to_string())),
        Color::DarkGray.paint(shorten_module_path(module_path, 35))
    )?;

    write!(w, "{}", style(level).paint(record.args().to_string()))
}