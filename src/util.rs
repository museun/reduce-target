use std::path::PathBuf;

pub fn humanize(d: u64) -> String {
    const SIZES: [char; 9] = ['B', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];

    let mut order = 0;
    let mut size = d as f64;
    while size >= 1024.0 && order + 1 < SIZES.len() {
        order += 1;
        size /= 1024.0;
    }

    format!("{:.2} {}", size, SIZES[order])
}

pub fn commaize(d: u64) -> String {
    use std::fmt::Write;
    fn comma(n: u64, s: &mut String) {
        if n < 1000 {
            write!(s, "{}", n).unwrap();
            return;
        }
        comma(n / 1000, s);
        write!(s, ",{:03}", n % 1000).unwrap();
    }

    let mut buf = String::new();
    comma(d, &mut buf);
    buf
}

pub fn fix_display_path(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    if cfg!(target_os = "windows") {
        path.display().to_string().replace(r"\\?\", "")
    } else {
        path.display().to_string()
    }
}
