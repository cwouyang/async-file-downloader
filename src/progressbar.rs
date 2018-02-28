use indicatif::{ProgressBar, ProgressStyle};

pub fn new(name: String, size: u64) -> ProgressBar {
    let bar = ProgressBar::new(size);
    let style = ProgressStyle::default_bar()
        .template(&format!("{:<12} [{{elapsed_precise}}] {{bar:.{}}} {{bytes:>8}}/{{total_bytes:>8}} \
            eta:{{eta:>4}} {{msg}}", name, "yello"))
        .progress_chars("=> ");
    bar.set_style(style);
    bar
}