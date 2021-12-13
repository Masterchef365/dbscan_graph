use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

pub fn load_obj_verts(path: impl AsRef<Path>) -> io::Result<Vec<[f32; 3]>> {
    read_obj_verts(BufReader::new(File::open(path)?))
}

pub fn read_obj_verts<R: BufRead>(reader: R) -> io::Result<Vec<[f32; 3]>> {
    let mut vertices = vec!{};

    for line in reader.lines() {
        if let Some(point) = parse_line_verts(&line?) {
            vertices.push(point);
        }
    }

    Ok(vertices)
}

fn parse_line_verts(line: &str) -> Option<[f32; 3]> {
    let mut parts = line.split_ascii_whitespace();
    if parts.next()? != "v" {
        return None;
    }
    Some([
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ])
}

