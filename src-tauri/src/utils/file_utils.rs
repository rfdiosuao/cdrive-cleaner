use std::path::Path;

pub fn calc_dir_size(path: &Path, size: &mut u64, count: &mut u64) {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                calc_dir_size(&p, size, count);
            } else if let Ok(meta) = entry.metadata() {
                *size += meta.len();
                *count += 1;
            }
        }
    }
}

pub fn format_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size_f = size as f64;
    if size_f >= GB {
        format!("{:.2} GB", size_f / GB)
    } else if size_f >= MB {
        format!("{:.2} MB", size_f / MB)
    } else if size_f >= KB {
        format!("{:.2} KB", size_f / KB)
    } else {
        format!("{} B", size)
    }
}
