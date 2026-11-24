//! Make each pipeline creation compile a shader the driver has not seen.
//!
//! Drivers and wgpu-hal cache compiled shaders by their input, so creating the same pipeline
//! repeatedly would measure the cache. Comments do not help, because naga drops them, and
//! neither do entry point names, which some drivers (KosmicKrisp, for one) rename. What reaches
//! the driver is a changed constant that feeds an output. The constant comes from a salt that
//! depends on the process, the iteration, the case and the stage, so cases with identical sources
//! differ too.
//!
//! - Shaders with an `override BENCH_SALT` get the salt for it. It is XORed into a live seed.
//! - A compute entry point gets a write of the salt to a new storage buffer binding.
//! - A vertex or fragment entry point returns its output changed by a tiny salted amount (a
//!   position or color component, or an integer output XORed with the salt).
//! - If none of that applies, the first float literal in the entry point gets extra digits, and
//!   as a last resort only the entry points are renamed.
//!
//! The `variation` field of a result is the weakest of these used for any stage of the case.

use crate::common::inspect;

pub struct Varied {
    pub source: String,
    pub constants: Vec<(String, f64)>,
    /// The entry point names to use, in the order they were given.
    pub entry_points: Vec<String>,
    /// `override`, `inject`, `literal` or `rename`.
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

fn fnv1a(hash: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(hash, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

/// The per-process part of every salt. `NAGA_PASSES_SALT_BASE` (any `u32`) fixes it, which makes
/// the shaders the same in every process.
fn base() -> u32 {
    static BASE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *BASE.get_or_init(|| {
        std::env::var("NAGA_PASSES_SALT_BASE")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or_else(|| {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos());
                let mixed = fnv1a(0xcbf29ce484222325, &nanos.to_le_bytes());
                let mixed = fnv1a(mixed, &std::process::id().to_le_bytes());
                (mixed ^ (mixed >> 32)) as u32
            })
    })
}

/// The salt for iteration `run` of the stage `stage` of the case `case`. It is stable for a given
/// process, and different for every combination, so two cases with the same source still compile
/// different shaders. The result is never zero.
pub fn salt(run: u32, case: &str, stage: &str) -> u32 {
    let mut hash = fnv1a(0xcbf29ce484222325, &base().to_le_bytes());
    hash = fnv1a(hash, &run.to_le_bytes());
    hash = fnv1a(hash, case.as_bytes());
    hash = fnv1a(hash, &[0xff]);
    hash = fnv1a(hash, stage.as_bytes());
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    ((hash ^ (hash >> 32)) as u32).max(1)
}

fn stage_name(stage: naga::ShaderStage) -> &'static str {
    match stage {
        naga::ShaderStage::Vertex => "vertex",
        naga::ShaderStage::Fragment => "fragment",
        naga::ShaderStage::Compute => "compute",
        naga::ShaderStage::Task => "task",
        naga::ShaderStage::Mesh => "mesh",
        _ => "other",
    }
}

/// How much a variation changes what the driver compiles, strongest first.
const KINDS: [&str; 4] = ["override", "inject", "literal", "rename"];

fn weakest(a: &'static str, b: &'static str) -> &'static str {
    let rank = |kind| KINDS.iter().position(|&k| k == kind).unwrap_or(KINDS.len());
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

/// The WGSL name of a 32-bit scalar or vector type, and what its elements are.
fn simple_type(inner: &naga::TypeInner) -> Option<(String, naga::ScalarKind)> {
    use naga::ScalarKind as K;
    let (size, scalar) = match *inner {
        naga::TypeInner::Scalar(scalar) => (1, scalar),
        naga::TypeInner::Vector { size, scalar } => (size as u32, scalar),
        _ => return None,
    };
    let element = match (scalar.kind, scalar.width) {
        (K::Float, 4) => "f32",
        (K::Sint, 4) => "i32",
        (K::Uint, 4) => "u32",
        _ => return None,
    };
    let name = if size == 1 {
        element.to_string()
    } else {
        format!("vec{size}<{element}>")
    };
    Some((name, scalar.kind))
}

/// An expression of type `type_name` that holds the salted constant of `tag`.
fn salted_term(type_name: &str, kind: naga::ScalarKind, tag: &str) -> String {
    let (element, constant) = match kind {
        naga::ScalarKind::Float => ("f32", format!("BENCH_SALT_F_{tag}")),
        naga::ScalarKind::Sint => ("i32", format!("BENCH_SALT_I_{tag}")),
        _ => ("u32", format!("BENCH_SALT_U_{tag}")),
    };
    if type_name == element {
        constant
    } else {
        format!("{type_name}({constant})")
    }
}

/// The salted constant of `tag` for this kind of value. The float one is tiny, so adding it
/// changes no visible output.
fn salt_constant(salt: u32, tag: &str, kind: naga::ScalarKind) -> String {
    match kind {
        naga::ScalarKind::Float => format!(
            "const BENCH_SALT_F_{tag}: f32 = f32({}u) * 1e-30;\n",
            salt & 0x00ff_ffff
        ),
        naga::ScalarKind::Sint => {
            format!("const BENCH_SALT_I_{tag}: i32 = {}i;\n", salt & 0x7fff_ffff)
        }
        _ => format!("const BENCH_SALT_U_{tag}: u32 = {salt}u;\n"),
    }
}

/// The end of `return ...;` whose expression starts at `start`.
fn statement_end(masked: &[u8], start: usize, end: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (offset, &c) in masked[start..end].iter().enumerate() {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b';' if depth == 0 => return Some(start + offset),
            _ => {}
        }
    }
    None
}

/// Replace every `return EXPR;` of the entry point with `replace(EXPR)`, which must end the
/// function itself.
fn rewrite_returns(text: &str, ep: &str, replace: impl Fn(&str) -> String) -> Option<String> {
    let masked = mask_comments(text);
    let (start, end) = function_body(&masked, ep)?;
    let mut out = String::with_capacity(text.len() + 128);
    let mut last = 0;
    let mut i = start;
    let mut found = false;
    while i < end {
        if is_ident(masked[i]) {
            let word_start = i;
            while i < end && is_ident(masked[i]) {
                i += 1;
            }
            if &masked[word_start..i] == b"return" {
                let semicolon = statement_end(&masked, i, end)?;
                let expr = text[i..semicolon].trim();
                if expr.is_empty() {
                    return None;
                }
                out.push_str(&text[last..word_start]);
                out.push_str(&replace(expr));
                last = semicolon + 1;
                i = last;
                found = true;
            }
        } else {
            i += 1;
        }
    }
    out.push_str(&text[last..]);
    found.then_some(out)
}

/// A vertex or fragment entry point whose returned value is changed by the salt: a position or
/// color component moves by a tiny amount, an integer output is XORed. This adds a few
/// instructions and no function, so passes that inline or reorder have nothing extra to do.
fn inject_output(
    text: &str,
    module: &naga::Module,
    entry: &naga::EntryPoint,
    ep: &str,
    salt: u32,
) -> Option<String> {
    let tag = stage_name(entry.stage);
    let result = entry.function.result.as_ref()?;
    let ty = &module.types[result.ty];
    let op = |kind| {
        if kind == naga::ScalarKind::Float {
            "+"
        } else {
            "^"
        }
    };
    let mut constant = String::new();
    let out = match (&result.binding, &ty.inner) {
        (Some(_), inner) => {
            let (name, kind) = simple_type(inner)?;
            let term = salted_term(&name, kind, tag);
            let op = op(kind);
            constant = salt_constant(salt, tag, kind);
            rewrite_returns(text, ep, |expr| format!("return ({expr}) {op} {term};"))
        }
        (None, naga::TypeInner::Struct { members, .. }) => {
            let want_position = entry.stage == naga::ShaderStage::Vertex;
            let member = members.iter().find(|m| {
                let simple = simple_type(&module.types[m.ty].inner).is_some();
                match m.binding {
                    Some(naga::Binding::BuiltIn(naga::BuiltIn::Position { .. })) => {
                        want_position && simple
                    }
                    Some(naga::Binding::Location { .. })
                    | Some(naga::Binding::BuiltIn(naga::BuiltIn::FragDepth)) => {
                        !want_position && simple
                    }
                    _ => false,
                }
            })?;
            let (name, kind) = simple_type(&module.types[member.ty].inner)?;
            let field = member.name.as_ref()?;
            let term = salted_term(&name, kind, tag);
            let op = op(kind);
            constant = salt_constant(salt, tag, kind);
            rewrite_returns(text, ep, |expr| {
                format!(
                    "{{ var bench_salt_r = {expr}; bench_salt_r.{field} = \
                     bench_salt_r.{field} {op} {term}; return bench_salt_r; }}"
                )
            })
        }
        _ => None,
    }?;
    Some(format!("{out}\n{constant}"))
}

/// A compute entry point that stores the salt to a storage buffer binding of its own.
fn inject_compute(text: &str, module: &naga::Module, ep: &str, salt: u32) -> Option<String> {
    let mut storage = 0;
    let mut group = u32::MAX;
    for (_, global) in module.global_variables.iter() {
        if let Some(binding) = &global.binding {
            group = group.min(binding.group);
        }
        if matches!(global.space, naga::AddressSpace::Storage { .. }) {
            storage += 1;
        }
    }
    if storage >= 8 {
        return None;
    }
    let group = if group == u32::MAX { 0 } else { group };
    let binding = module
        .global_variables
        .iter()
        .filter_map(|(_, g)| g.binding.as_ref())
        .filter(|b| b.group == group)
        .map(|b| b.binding + 1)
        .max()
        .unwrap_or(0);

    let masked = mask_comments(text);
    let (start, _) = function_body(&masked, ep)?;
    let mut out = String::with_capacity(text.len() + 256);
    out.push_str(&text[..start]);
    out.push_str("\n    bench_salt_sink.v = BENCH_SALT_U_compute;");
    out.push_str(&text[start..]);
    out.push('\n');
    out.push_str(&salt_constant(salt, "compute", naga::ScalarKind::Uint));
    out.push_str(&format!(
        "struct BenchSaltSink {{ v: u32 }}\n\
         @group({group}) @binding({binding}) var<storage, read_write> bench_salt_sink: BenchSaltSink;\n"
    ));
    Some(out)
}

/// Extra digits on the first float literal of the entry point.
fn inject_literal(text: &str, ep: &str, salt: u32) -> Option<String> {
    let masked = mask_comments(text);
    let range = function_body(&masked, ep)?;
    let position = first_float_literal(&masked, range)?;
    let mut out = text.to_string();
    out.insert_str(position, &format!("{:04}", salt % 10000));
    Some(out)
}

/// Whether the varied source is still a valid module, if the original is.
fn accepts(text: &str, original_valid: bool) -> bool {
    let Ok(module) = inspect::parse(text) else {
        return false;
    };
    !original_valid || validates(&module)
}

fn validates(module: &naga::Module) -> bool {
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(module)
    .is_ok()
}

/// `source` for iteration `run` of the case `case`, with a different salt for each of the stages
/// of `entry_points`.
pub fn vary(case: &str, source: &str, entry_points: &[&str], run: u32) -> Varied {
    if source.contains("override BENCH_SALT") {
        return Varied {
            source: source.to_string(),
            constants: vec![(
                "BENCH_SALT".to_string(),
                f64::from(salt(run, case, "compute")),
            )],
            entry_points: entry_points.iter().map(|ep| ep.to_string()).collect(),
            kind: "override",
        };
    }

    let module = inspect::parse(source).ok();
    let original_valid = module.as_ref().is_some_and(validates);
    let mut text = source.to_string();
    let mut kind = "inject";
    for ep in entry_points {
        let entry = module
            .as_ref()
            .and_then(|m| m.entry_points.iter().find(|e| e.name == *ep));
        let salt = salt(run, case, entry.map_or("unknown", |e| stage_name(e.stage)));

        let injected = match (&module, entry) {
            (Some(module), Some(entry)) => match entry.stage {
                naga::ShaderStage::Compute => inject_compute(&text, module, ep, salt),
                naga::ShaderStage::Vertex | naga::ShaderStage::Fragment => {
                    inject_output(&text, module, entry, ep, salt)
                }
                _ => None,
            },
            _ => None,
        }
        .filter(|candidate| accepts(candidate, original_valid));
        if let Some(candidate) = injected {
            text = candidate;
            continue;
        }

        let literal =
            inject_literal(&text, ep, salt).filter(|candidate| accepts(candidate, original_valid));
        match literal {
            Some(candidate) => {
                text = candidate;
                kind = weakest(kind, "literal");
            }
            None => kind = weakest(kind, "rename"),
        }
    }

    let mut renamed = Vec::new();
    for ep in entry_points {
        let new = format!("{ep}_s{}", salt(run, case, "name"));
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
