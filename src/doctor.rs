/// Self-check / doctor mode.
///
/// Verifies:
///   1. Typst engine: compile a tiny document round-trip
///   2. Embedded fonts: confirm the default body and mono fonts are found
///   3. Cache directory: writable scratch space
///   4. Network: reachable for remote-image downloads (optional)
///
/// Exits with code 0 on full pass, 1 if any check fails.
use std::path::PathBuf;

use anyhow::Result;
#[allow(unused_imports)]
use typst::LibraryExt;

// ─────────────────────────────────────────────────────────────────────────────

struct Check {
    label: &'static str,
    result: CheckResult,
}

enum CheckResult {
    Pass(String),
    Warn(String),
    Fail(String),
}

impl CheckResult {
    fn icon(&self) -> &'static str {
        match self {
            CheckResult::Pass(_) => "✅",
            CheckResult::Warn(_) => "⚠️ ",
            CheckResult::Fail(_) => "❌",
        }
    }
    fn detail(&self) -> &str {
        match self {
            CheckResult::Pass(s) | CheckResult::Warn(s) | CheckResult::Fail(s) => s,
        }
    }
    fn is_fail(&self) -> bool {
        matches!(self, CheckResult::Fail(_))
    }
}

// ─────────────────────────────────────────────────────────────────────────────

pub fn run() -> Result<()> {
    eprintln!();
    eprintln!("  md-to-pdf — doctor / self-check");
    eprintln!("  ─────────────────────────────────");

    let checks = vec![
        // 1. Typst engine round-trip
        check_typst_engine(),
        // 2. Default body font
        check_font("Libertinus Serif", "body (default)"),
        // 3. Default mono font
        check_font("DejaVu Sans Mono", "mono (default)"),
        // 4. Cache directory writability
        check_cache_dir(),
        // 5. Network reachability
        check_network(),
        // 6. Rust toolchain (just informational)
        check_rust_version(),
    ];

    // ── print table ───────────────────────────────────────────────────────
    eprintln!();
    let mut any_fail = false;
    for c in &checks {
        eprintln!(
            "  {} {:<30}  {}",
            c.result.icon(),
            c.label,
            c.result.detail()
        );
        if c.result.is_fail() {
            any_fail = true;
        }
    }
    eprintln!();

    if any_fail {
        eprintln!("  Some checks failed — see details above.");
        eprintln!();
        std::process::exit(1);
    } else {
        eprintln!("  All checks passed. md-to-pdf is ready to use.");
        eprintln!();
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Individual checks
// ─────────────────────────────────────────────────────────────────────────────

fn check_typst_engine() -> Check {
    use std::collections::HashMap;
    use typst::foundations::{Bytes, Datetime, Smart};
    use typst::layout::PagedDocument;
    use typst::text::{Font, FontBook};
    use typst::utils::LazyHash;
    use typst::{Library, LibraryExt, World, compile};
    use typst_kit::fonts::{FontSearcher, Fonts};
    use typst_pdf::{PdfOptions, pdf};
    use typst_syntax::{FileId, Source, VirtualPath};

    struct MiniWorld {
        library: LazyHash<Library>,
        book: LazyHash<FontBook>,
        fonts: Vec<typst_kit::fonts::FontSlot>,
        main: FileId,
        sources: HashMap<FileId, Source>,
    }

    impl World for MiniWorld {
        fn library(&self) -> &LazyHash<Library> {
            &self.library
        }
        fn book(&self) -> &LazyHash<FontBook> {
            &self.book
        }
        fn main(&self) -> FileId {
            self.main
        }
        fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
            self.sources.get(&id).cloned().ok_or_else(|| {
                typst::diag::FileError::NotFound(id.vpath().as_rootless_path().to_path_buf())
            })
        }
        fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rootless_path().to_path_buf(),
            ))
        }
        fn font(&self, index: usize) -> Option<Font> {
            self.fonts.get(index).and_then(|s| s.get())
        }
        fn today(&self, _: Option<i64>) -> Option<Datetime> {
            None
        }
    }

    let mut searcher = FontSearcher::new();
    let Fonts { book, fonts } = searcher.search();
    let main_id = FileId::new(None, VirtualPath::new("/doctor-check.typ"));
    let mut sources = HashMap::new();
    sources.insert(
        main_id,
        Source::new(main_id, "= Doctor check\nHello, world!".to_string()),
    );

    let world = MiniWorld {
        library: LazyHash::new(Library::default()),
        book: LazyHash::new(book),
        fonts,
        main: main_id,
        sources,
    };

    let warned = compile::<PagedDocument>(&world);
    match warned.output {
        Ok(doc) => {
            let pages = doc.pages.len();
            match pdf(
                &doc,
                &PdfOptions {
                    ident: Smart::Auto,
                    ..PdfOptions::default()
                },
            ) {
                Ok(bytes) => Check {
                    label: "Typst engine",
                    result: CheckResult::Pass(format!("{} page(s), {} bytes", pages, bytes.len())),
                },
                Err(e) => Check {
                    label: "Typst engine",
                    result: CheckResult::Fail(format!("PDF render error: {:?}", e)),
                },
            }
        }
        Err(e) => Check {
            label: "Typst engine",
            result: CheckResult::Fail(format!("compile error: {:?}", e)),
        },
    }
}

fn check_font(family: &'static str, role: &'static str) -> Check {
    use typst_kit::fonts::{FontSearcher, Fonts};

    let mut searcher = FontSearcher::new();
    let Fonts { book, .. } = searcher.search();

    let label = Box::leak(format!("Font: {family} ({role})").into_boxed_str());
    let found = book.select(family, Default::default()).is_some();

    if found {
        Check {
            label,
            result: CheckResult::Pass("found".to_string()),
        }
    } else {
        Check {
            label,
            result: CheckResult::Warn(format!(
                "not found — Typst will substitute a fallback. \
Install \"{family}\" for best results."
            )),
        }
    }
}

fn check_cache_dir() -> Check {
    let label = "Cache directory writable";

    let dir: PathBuf = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".md-to-pdf-cache");

    match std::fs::create_dir_all(&dir) {
        Err(e) => Check {
            label,
            result: CheckResult::Fail(format!("cannot create {}: {}", dir.display(), e)),
        },
        Ok(()) => {
            let probe = dir.join(".doctor-probe");
            match std::fs::write(&probe, b"ok") {
                Ok(()) => {
                    let _ = std::fs::remove_file(&probe);
                    Check {
                        label,
                        result: CheckResult::Pass(dir.display().to_string()),
                    }
                }
                Err(e) => Check {
                    label,
                    result: CheckResult::Fail(format!("{}: {}", dir.display(), e)),
                },
            }
        }
    }
}

fn check_network() -> Check {
    let label = "Network (image caching)";
    let url = "https://www.google.com";

    match ureq::get(url).call() {
        Ok(_) => Check {
            label,
            result: CheckResult::Pass("reachable".to_string()),
        },
        Err(e) => Check {
            label,
            result: CheckResult::Warn(format!(
                "unreachable ({e}) — remote images will fail; use --no-remote-images to suppress"
            )),
        },
    }
}

fn check_rust_version() -> Check {
    let label = "Rust toolchain";
    // VERGEN is not available here; use the compile-time env fallback.
    let version = option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("unknown");
    Check {
        label,
        result: CheckResult::Pass(format!("built with MSRV {}", version)),
    }
}
