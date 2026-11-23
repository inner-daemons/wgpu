//! Make each pipeline creation compile a shader the driver has not seen.
//!
//! Drivers and wgpu-hal cache compiled shaders by their input, so creating the same pipeline
//! repeatedly would measure the cache. Comments do not help, because naga drops them. What does
//! reach the driver is a changed constant, so:
//!
//! - Shaders with an `override BENCH_SALT` get a different value for it each time.
//! - For other shaders, the first float literal in the body of each entry point gets extra
//!   digits, and the entry points are renamed. Renaming alone changes the SPIR-V, MSL and HLSL
//!   text, but a driver that hashes its optimized IR may still hit a cache.

pub struct Varied {
    pub source: String,
    pub constants: Vec<(String, f64)>,
    /// The entry point names to use, in the order they were given.
    pub entry_points: Vec<String>,
    /// `override`, `literal` or `rename`.
    pub kind: &'static str,
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The source with comments blanked out, so searches skip them.
fn mask_comments(source: &str) -> Vec<u8> {
    let mut b = source.as_bytes().to_vec();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                b[i] = b' ';
                i += 1;
            }
        } else if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 0;
            while i < b.len() {
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    b[i] = b' ';
                    b[i + 1] = b' ';
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    b[i] = b' ';
                    b[i + 1] = b' ';
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if b[i] != b'\n' {
                        b[i] = b' ';
                    }
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    b
}

/// The byte range of the body of `fn name`, between its braces.
fn function_body(masked: &[u8], name: &str) -> Option<(usize, usize)> {
    let mut i = 0;
    while i + 2 < masked.len() {
        let at_fn = &masked[i..i + 2] == b"fn"
            && (i == 0 || !is_ident(masked[i - 1]))
            && masked[i + 2].is_ascii_whitespace();
        if at_fn {
            let mut j = i + 2;
            while masked[j].is_ascii_whitespace() {
                j += 1;
            }
            let start = j;
            while j < masked.len() && is_ident(masked[j]) {
                j += 1;
            }
            if &masked[start..j] == name.as_bytes() {
                let open = j + masked[j..].iter().position(|&c| c == b'{')?;
                let mut depth = 0;
                for (k, &c) in masked[open..].iter().enumerate() {
                    match c {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some((open + 1, open + k));
                            }
                        }
                        _ => {}
                    }
                }
                return None;
            }
        }
        i += 1;
    }
    None
}

/// The position after the first fractional digit of the first decimal float literal in `range`.
fn first_float_literal(masked: &[u8], (start, end): (usize, usize)) -> Option<usize> {
    let mut i = start;
    while i < end {
        let c = masked[i];
        if is_ident(c) && !c.is_ascii_digit() {
            while i < end && is_ident(masked[i]) {
                i += 1;
            }
        } else if c.is_ascii_digit() && (i == 0 || masked[i - 1] != b'.') {
            while i < end && masked[i].is_ascii_digit() {
                i += 1;
            }
            if i < end && masked[i] == b'.' {
                let fraction = i + 1;
                let mut j = fraction;
                while j < end && masked[j].is_ascii_digit() {
                    j += 1;
                }
                let next = masked.get(j).copied().unwrap_or(b' ');
                if j > fraction && (next == b'f' || !is_ident(next)) {
                    return Some(fraction + 1);
                }
                i = j;
            }
        } else {
            i += 1;
        }
    }
    None
}

fn replace_ident(source: &str, from: &str, to: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len() + 16);
    let mut last = 0;
    let mut i = 0;
    while i < bytes.len() {
        if is_ident(bytes[i]) {
            let start = i;
            while i < bytes.len() && is_ident(bytes[i]) {
                i += 1;
            }
            if &source[start..i] == from {
                out.push_str(&source[last..start]);
                out.push_str(to);
                last = i;
            }
        } else {
            i += 1;
        }
    }
    out.push_str(&source[last..]);
    out
}

pub fn vary(source: &str, entry_points: &[&str], salt: u32) -> Varied {
    if source.contains("override BENCH_SALT") {
        return Varied {
            source: source.to_string(),
            constants: vec![("BENCH_SALT".to_string(), f64::from(salt))],
            entry_points: entry_points.iter().map(|ep| ep.to_string()).collect(),
            kind: "override",
        };
    }

    let mut text = source.to_string();
    let mut kind = "rename";
    for ep in entry_points {
        let masked = mask_comments(&text);
        let position =
            function_body(&masked, ep).and_then(|range| first_float_literal(&masked, range));
        if let Some(position) = position {
            text.insert_str(position, &format!("{salt:03}"));
            kind = "literal";
        }
    }

    let mut renamed = Vec::new();
    for ep in entry_points {
        let new = format!("{ep}_s{salt}");
        text = replace_ident(&text, ep, &new);
        renamed.push(new);
    }
    Varied {
        source: text,
        constants: Vec::new(),
        entry_points: renamed,
        kind,
    }
}
