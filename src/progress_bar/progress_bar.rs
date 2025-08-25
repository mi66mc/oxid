use crate::print_top;
use crate::string::string::String;


pub fn progress_bar(all: usize, progress: usize, width: usize) {
    let mut bar = String::new();
    let filled = progress * width / all;

    for _ in 0..filled {
        bar.push_str("|");
    }

    let empty = width - filled;
    for _ in 0..empty {
        bar.push_str(" ");
    }

    print_top!("[{}] {}%\r", bar.as_str(), progress * 100 / all);
}