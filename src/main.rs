//! `oovra` — CLI front-end.
//!
//! Four subcommands:
//!   - create    Scaffold a new atom or label an existing .md file
//!   - compose   Compose ordered inputs into a compound
//!   - decompose Recover a compound's inputs (one level, or --full)
//!   - compare   Diff two prompt elements (kind-aware)

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use oovra::create::{copy_oovra_into_olib, ensure_dir, label_in_place, label_into_olib};
use oovra::decompose::{decompose_full, report};
use oovra::diff::{compare, DiffReport};
use oovra::discovery::discover;
use oovra::element::{
    looks_like_oovra_file, parse, parse_file_with, serialize, write, ParseOptions, PromptElement,
};
use oovra::header::{is_kebab_case, slugify};
use oovra::library::Library;
use oovra::migrate::migrate_library;
use oovra::render::{compose, render_text, ComposeRequest};

#[derive(Parser, Debug)]
#[command(name = "oovra", version, about = "Compose and compare agentic prompt elements (Markdown + TOML).", long_about = None)]
struct Cli {
    /// Accept v0.1 legacy schema files (with `order` instead of `kind`).
    /// Read-only ergonomics during the v0.2 transition; writes are always
    /// in v0.2 format. Removed in v0.3.
    #[arg(long, global = true)]
    legacy: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create a new prompt element (always an atom)
    Create(CreateArgs),

    /// Compose prompt elements into a compound
    Compose(ComposeArgs),

    /// Decompose a compound into its inputs
    Decompose(DecomposeArgs),

    /// Compare two prompt elements
    Compare(CompareArgs),

    /// Discover olib directories under a filesystem root
    Discover(DiscoverArgs),

    /// Inspect a single prompt element file (read-only)
    Inspect(InspectArgs),

    /// Migrate a v0.1 library to v0.2 schema in place
    Migrate(MigrateArgs),
}

#[derive(clap::Args, Debug)]
struct CreateArgs {
    /// Header these files IN PLACE — each file becomes an Oovra element.
    /// Accepts files and/or directories (a directory contributes the .md
    /// files directly inside it). Mutually exclusive with --olib.
    #[arg(long, value_name = "PATH", num_args = 1.., group = "mode")]
    label: Option<Vec<PathBuf>>,

    /// Copy these files into the olib library as headered Oovra elements,
    /// leaving the originals untouched. Accepts files and/or directories.
    /// An input that is already an Oovra file is copied verbatim
    /// (olib-to-olib transfer). Mutually exclusive with --label.
    #[arg(long, value_name = "PATH", num_args = 1.., group = "mode")]
    olib: Option<Vec<PathBuf>>,

    /// Olib library directory for --olib (created if missing).
    #[arg(long, default_value = "./olib")]
    library: PathBuf,

    /// Semver version recorded on newly headered elements.
    #[arg(long, default_value = "1.0.0")]
    version: String,

    /// Meta description recorded on newly headered elements.
    #[arg(long, default_value = "")]
    meta: String,

    /// Auto-slugify non-kebab-case filenames into ids without prompting.
    #[arg(long)]
    slug: bool,

    /// Relabel a file that already carries an Oovra header (--label only).
    #[arg(long)]
    force: bool,
}

#[derive(clap::Args, Debug)]
struct ComposeArgs {
    /// Input element IDs in order
    #[arg(required_unless_present = "re_render")]
    ids: Vec<String>,

    /// Path to write the composed element
    #[arg(short, long, conflicts_with_all = ["text", "re_render"])]
    output: Option<PathBuf>,

    /// Output element ID (defaults to a kebab-case name based on inputs)
    #[arg(long)]
    out_id: Option<String>,

    /// Output element name (defaults to the output ID)
    #[arg(long)]
    out_name: Option<String>,

    /// Output element version
    #[arg(long, default_value = "1.0.0")]
    out_version: String,

    /// Output element meta description
    #[arg(long, default_value = "")]
    out_meta: String,

    /// Library directory to resolve inputs from
    #[arg(long, default_value = "./olib")]
    library: PathBuf,

    /// Print the rendered body to stdout instead of writing a file
    #[arg(long)]
    text: bool,

    /// Re-render an existing composed file against the current library
    #[arg(long, value_name = "PATH")]
    re_render: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct DecomposeArgs {
    /// Path to a compound (one that has a `composed_of` recipe)
    path: PathBuf,

    /// Recursively decompose to atom leaves and write a folder tree
    #[arg(long)]
    full: bool,

    /// Output directory (used with --full). Defaults to the current dir.
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    /// Format for non-full mode: human or json
    #[arg(long, default_value = "human")]
    format: String,
}

#[derive(clap::Args, Debug)]
struct CompareArgs {
    /// First file
    a: PathBuf,
    /// Second file
    b: PathBuf,
    /// Format: human or json
    #[arg(long, default_value = "human")]
    format: String,
}

#[derive(clap::Args, Debug)]
struct DiscoverArgs {
    /// Filesystem root to walk
    root: PathBuf,

    /// Optional depth bound (root itself is depth 0; no limit by default)
    #[arg(long)]
    max_depth: Option<usize>,

    /// Output format: human (default) or json
    #[arg(long, default_value = "human")]
    format: String,
}

#[derive(clap::Args, Debug)]
struct InspectArgs {
    /// Path to the prompt-element file (atom or compound)
    path: PathBuf,

    /// Output format: human (default) or json
    #[arg(long, default_value = "human")]
    format: String,
}

#[derive(clap::Args, Debug)]
struct MigrateArgs {
    /// Library directory to migrate in place. Recursive. Run in a clean
    /// Git working directory so the diff is auditable.
    library: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let opts = ParseOptions { legacy: cli.legacy };
    match cli.command {
        Command::Create(args) => run_create(args),
        Command::Compose(args) => run_compose(args, opts),
        Command::Decompose(args) => run_decompose(args, opts),
        Command::Compare(args) => run_compare(args, opts),
        Command::Discover(args) => run_discover(args),
        Command::Inspect(args) => run_inspect(args, opts),
        Command::Migrate(args) => run_migrate(args),
    }
}

/// Which `create` mode the user selected.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CreateMode {
    /// Header files in place — the file becomes the element.
    Label,
    /// Copy headered elements into the olib library.
    Olib,
}

/// The outcome of processing one input file.
enum FileOutcome {
    /// An element was written at this path.
    Written(PathBuf),
    /// The file was deliberately not processed, with a reason.
    Skipped(String),
}

/// Resolution of the kebab-case id for a plain input file.
enum IdResolution {
    /// Use this id; if `rename_to` is set, rename the source file first.
    Use {
        id: String,
        rename_to: Option<PathBuf>,
    },
    /// Skip this file, with a human-readable reason.
    Skip(String),
}

fn run_create(args: CreateArgs) -> anyhow::Result<()> {
    let (mode, input_paths) = match (&args.label, &args.olib) {
        (Some(paths), None) => (CreateMode::Label, paths.clone()),
        (None, Some(paths)) => (CreateMode::Olib, paths.clone()),
        (None, None) => {
            return Err(anyhow!(
                "pass --label <path>... (header files in place) or \
                 --olib <path>... (copy headered elements into ./olib/)"
            ))
        }
        (Some(_), Some(_)) => unreachable!("clap 'mode' group forbids both"),
    };

    let (files, mut failures) = collect_input_files(&input_paths);
    if files.is_empty() && failures.is_empty() {
        return Err(anyhow!("no .md files found in the given path(s)"));
    }

    if mode == CreateMode::Olib {
        ensure_dir(&args.library)
            .with_context(|| format!("preparing olib directory {}", args.library.display()))?;
    }

    let mut written: Vec<PathBuf> = Vec::new();
    let mut skipped: Vec<(PathBuf, String)> = Vec::new();

    for file in &files {
        match process_one(file, mode, &args) {
            Ok(FileOutcome::Written(p)) => written.push(p),
            Ok(FileOutcome::Skipped(reason)) => skipped.push((file.clone(), reason)),
            Err(e) => failures.push((file.clone(), format!("{e:#}"))),
        }
    }

    println!(
        "{} {} written, {} skipped, {} failed",
        "Create".green().bold(),
        written.len(),
        skipped.len(),
        failures.len()
    );
    for p in &written {
        println!("  {} {}", "✓".green(), p.display());
    }
    for (p, reason) in &skipped {
        println!("  {} {} ({})", "-".dimmed(), p.display(), reason);
    }
    for (p, err) in &failures {
        eprintln!("  {} {}: {}", "✗".red(), p.display(), err);
    }
    if !failures.is_empty() {
        anyhow::bail!("{} file(s) failed to process", failures.len());
    }
    Ok(())
}

/// Expand the user-given paths into a flat list of `.md` files. A directory
/// contributes the `.md` files directly inside it (non-recursive). Paths
/// that don't exist, or directories that can't be read, become failures.
fn collect_input_files(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<(PathBuf, String)>) {
    let mut files = Vec::new();
    let mut failures = Vec::new();
    for p in paths {
        if !p.exists() {
            failures.push((p.clone(), "path does not exist".to_string()));
        } else if p.is_dir() {
            match std::fs::read_dir(p) {
                Ok(entries) => {
                    let mut md: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|c| c.is_file() && has_md_extension(c))
                        .collect();
                    md.sort();
                    files.extend(md);
                }
                Err(e) => failures.push((p.clone(), format!("cannot read directory: {e}"))),
            }
        } else {
            files.push(p.clone());
        }
    }
    (files, failures)
}

/// Case-insensitive check for a `.md` extension.
fn has_md_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

/// Process one input file according to the selected mode.
fn process_one(file: &Path, mode: CreateMode, args: &CreateArgs) -> anyhow::Result<FileOutcome> {
    let content =
        std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let is_oovra = looks_like_oovra_file(&content);

    match mode {
        CreateMode::Olib => {
            if is_oovra {
                // olib-to-olib transfer: copy verbatim, keep the one header.
                let dest = copy_oovra_into_olib(&args.library, file, &content)?;
                return Ok(FileOutcome::Written(dest));
            }
            let (id, content) = match resolve_id(file, args.slug)? {
                IdResolution::Skip(reason) => return Ok(FileOutcome::Skipped(reason)),
                IdResolution::Use {
                    id,
                    rename_to: None,
                } => (id, content),
                IdResolution::Use {
                    id,
                    rename_to: Some(renamed),
                } => {
                    rename_source(file, &renamed)?;
                    (id, std::fs::read_to_string(&renamed)?)
                }
            };
            let dest = label_into_olib(&args.library, &content, &id, &args.version, &args.meta)?;
            Ok(FileOutcome::Written(dest))
        }
        CreateMode::Label => {
            if is_oovra {
                if !args.force {
                    return Ok(FileOutcome::Skipped(
                        "already an Oovra file — pass --force to relabel".to_string(),
                    ));
                }
                // Force-relabel keeps the element's existing id.
                let existing = parse(&content, file)
                    .with_context(|| format!("parsing {} for relabel", file.display()))?;
                let dest = label_in_place(
                    file,
                    &content,
                    &existing.header.id,
                    &args.version,
                    &args.meta,
                    true,
                )?;
                return Ok(FileOutcome::Written(dest));
            }
            let (id, source, content) = match resolve_id(file, args.slug)? {
                IdResolution::Skip(reason) => return Ok(FileOutcome::Skipped(reason)),
                IdResolution::Use {
                    id,
                    rename_to: None,
                } => (id, file.to_path_buf(), content),
                IdResolution::Use {
                    id,
                    rename_to: Some(renamed),
                } => {
                    rename_source(file, &renamed)?;
                    let c = std::fs::read_to_string(&renamed)?;
                    (id, renamed, c)
                }
            };
            let dest = label_in_place(&source, &content, &id, &args.version, &args.meta, false)?;
            Ok(FileOutcome::Written(dest))
        }
    }
}

/// Rename a source file, failing if the destination already exists.
fn rename_source(from: &Path, to: &Path) -> anyhow::Result<()> {
    if to.exists() {
        return Err(anyhow!(
            "cannot rename {} -> {}: destination already exists",
            from.display(),
            to.display()
        ));
    }
    std::fs::rename(from, to)
        .with_context(|| format!("renaming {} -> {}", from.display(), to.display()))
}

/// Resolve the kebab-case id for a plain input file from its filename stem.
///
/// A kebab-case stem is used as-is. Otherwise the stem is slugified, and
/// the choice of what to do is made by `--slug`, by an interactive prompt
/// when stdin is a terminal, or — when neither applies — by skipping the
/// file with advice.
fn resolve_id(source: &Path, slug_flag: bool) -> anyhow::Result<IdResolution> {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    if is_kebab_case(stem) {
        return Ok(IdResolution::Use {
            id: stem.to_string(),
            rename_to: None,
        });
    }

    let slug = match slugify(stem) {
        Some(s) => s,
        None => {
            return Ok(IdResolution::Skip(format!(
                "filename {stem:?} has no characters usable for a kebab-case id"
            )))
        }
    };

    if slug_flag {
        return Ok(IdResolution::Use {
            id: slug,
            rename_to: None,
        });
    }

    if !std::io::stdin().is_terminal() {
        return Ok(IdResolution::Skip(format!(
            "filename {stem:?} isn't kebab-case — rerun with --slug, or rename it to '{slug}.md'"
        )));
    }

    println!(
        "  {}: filename isn't kebab-case, so it can't be the element id.",
        source.display()
    );
    if prompt_yes_no(&format!("  Use the slug '{slug}' as the id?"))? {
        return Ok(IdResolution::Use {
            id: slug,
            rename_to: None,
        });
    }
    let renamed = source.with_file_name(format!("{slug}.md"));
    if prompt_yes_no(&format!(
        "  Rename the file {} -> {}?",
        source.display(),
        renamed.display()
    ))? {
        return Ok(IdResolution::Use {
            id: slug,
            rename_to: Some(renamed),
        });
    }
    Ok(IdResolution::Skip(format!(
        "filename {stem:?} isn't kebab-case — rename it (e.g. '{slug}.md') to include it"
    )))
}

/// Ask a yes/no question on the terminal, repeating until a clear answer
/// is given. EOF is treated as "no".
fn prompt_yes_no(question: &str) -> anyhow::Result<bool> {
    loop {
        print!("{question} [y/n] ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line)? == 0 {
            return Ok(false); // EOF
        }
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("  please answer 'y' or 'n'."),
        }
    }
}

fn run_compose(args: ComposeArgs, opts: ParseOptions) -> anyhow::Result<()> {
    let library = Library::load_with(&args.library, opts)
        .with_context(|| format!("loading library from {}", args.library.display()))?;

    if let Some(re_render_path) = args.re_render {
        let existing = parse_file_with(&re_render_path, opts)
            .with_context(|| format!("reading re-render target {}", re_render_path.display()))?;
        let composed_of = existing
            .header
            .composed_of
            .as_ref()
            .ok_or_else(|| anyhow!("--re-render target is order 0 and has no composed_of"))?
            .clone();

        let inputs: Vec<(String, Option<String>)> = composed_of
            .iter()
            .map(|i| (i.id.clone(), Some(i.version.clone())))
            .collect();

        let req = ComposeRequest {
            library: &library,
            inputs,
            output_id: existing.header.id.clone(),
            output_name: existing.header.name.clone(),
            output_version: existing.header.version.clone(),
            output_meta: existing.header.meta.clone(),
        };
        let composed = compose(req)?;
        write(&composed, &re_render_path)?;
        println!(
            "{} {} (body_level {})",
            "Re-rendered".green().bold(),
            re_render_path.display(),
            composed.header.body_level.unwrap_or(0)
        );
        return Ok(());
    }

    let inputs: Vec<(String, Option<String>)> =
        args.ids.iter().map(|id| (id.clone(), None)).collect();

    if args.text {
        let resolved: Vec<&PromptElement> = args
            .ids
            .iter()
            .map(|id| {
                library
                    .get(id)
                    .ok_or_else(|| anyhow!("element '{id}' not found in library"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let body = render_text(&resolved)?;
        print!("{body}");
        if !body.ends_with('\n') {
            println!();
        }
        return Ok(());
    }

    let out_id = args
        .out_id
        .clone()
        .unwrap_or_else(|| format!("composed-{}", args.ids.join("-")));
    let out_name = args.out_name.clone().unwrap_or_else(|| out_id.clone());

    let req = ComposeRequest {
        library: &library,
        inputs,
        output_id: out_id.clone(),
        output_name: out_name,
        output_version: args.out_version,
        output_meta: args.out_meta,
    };
    let composed = compose(req)?;
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| args.library.join(format!("{out_id}.md")));
    write(&composed, &output_path)?;
    println!(
        "{} {} (compound, body_level {}, {} inputs)",
        "Composed".green().bold(),
        output_path.display(),
        composed.header.body_level.unwrap_or(0),
        composed
            .header
            .composed_of
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0)
    );
    Ok(())
}

fn run_decompose(args: DecomposeArgs, opts: ParseOptions) -> anyhow::Result<()> {
    let element = parse_file_with(&args.path, opts)
        .with_context(|| format!("reading {}", args.path.display()))?;

    if args.full {
        let out_dir = decompose_full(&element, &args.output)?;
        println!(
            "{} {} -> {}",
            "Decomposed (full)".green().bold(),
            args.path.display(),
            out_dir.display()
        );
        return Ok(());
    }

    let report_data = report(&element)?;
    if args.format == "json" {
        let s = serde_json::to_string_pretty(&report_data)?;
        println!("{s}");
    } else {
        println!(
            "{} {} ({}) — {} at body_level {}, {} immediate input(s)",
            "Decompose".bold(),
            report_data.element_id.cyan(),
            report_data.element_version.dimmed(),
            report_data.element_kind,
            report_data.body_level,
            report_data.inputs.len()
        );
        for entry in &report_data.inputs {
            println!(
                "  - {} {} ({}) {}",
                entry.id.cyan(),
                format!("@ {}", entry.version).dimmed(),
                entry.kind,
                if !entry.name.is_empty() && entry.name != entry.id {
                    format!("— {}", entry.name)
                } else {
                    String::new()
                }
            );
        }
    }
    Ok(())
}

fn run_compare(args: CompareArgs, opts: ParseOptions) -> anyhow::Result<()> {
    let a =
        parse_file_with(&args.a, opts).with_context(|| format!("reading {}", args.a.display()))?;
    let b =
        parse_file_with(&args.b, opts).with_context(|| format!("reading {}", args.b.display()))?;
    let report = compare(&a, &b)?;

    if args.format == "json" {
        let s = serde_json::to_string_pretty(&report)?;
        println!("{s}");
        return Ok(());
    }

    match report {
        DiffReport::Content(c) => {
            println!(
                "{} {} <-> {}  (atoms, content diff)",
                "Compare".bold(),
                c.a_id.cyan(),
                c.b_id.cyan()
            );
            if c.field_changes.is_empty() {
                println!("  metadata: {}", "unchanged".dimmed());
            } else {
                println!("  metadata changes:");
                for fc in &c.field_changes {
                    println!(
                        "    {} : {} -> {}",
                        fc.field.yellow(),
                        fc.before.red(),
                        fc.after.green()
                    );
                }
            }
            if c.bodies_equal {
                println!("  body: {}", "unchanged".dimmed());
            } else {
                println!("  body diff:");
                for line in c.body_unified_diff.lines() {
                    if line.starts_with('+') && !line.starts_with("+++") {
                        println!("    {}", line.green());
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        println!("    {}", line.red());
                    } else if line.starts_with("@@") {
                        println!("    {}", line.cyan());
                    } else {
                        println!("    {line}");
                    }
                }
            }
        }
        DiffReport::Structural(s) => {
            println!(
                "{} {} <-> {}  (compounds, structural diff)",
                "Compare".bold(),
                s.a_id.cyan(),
                s.b_id.cyan(),
            );
            if s.recipes_equal {
                println!("  recipes: {}", "identical".dimmed());
            } else {
                if !s.added.is_empty() {
                    println!("  added inputs:");
                    for pi in &s.added {
                        println!(
                            "    {} [{}] {}",
                            "+".green(),
                            pi.position,
                            format!("{} @ {}", pi.input.id, pi.input.version).green()
                        );
                    }
                }
                if !s.removed.is_empty() {
                    println!("  removed inputs:");
                    for pi in &s.removed {
                        println!(
                            "    {} [{}] {}",
                            "-".red(),
                            pi.position,
                            format!("{} @ {}", pi.input.id, pi.input.version).red()
                        );
                    }
                }
                if !s.version_changed.is_empty() {
                    println!("  version-changed inputs:");
                    for v in &s.version_changed {
                        println!(
                            "    {} {} : {} -> {}",
                            "~".yellow(),
                            v.id.cyan(),
                            v.before_version.red(),
                            v.after_version.green()
                        );
                    }
                }
                if !s.moved.is_empty() {
                    println!("  moved inputs:");
                    for m in &s.moved {
                        println!(
                            "    {} {} @ {} : pos {} -> pos {}",
                            "↔".blue(),
                            m.id.cyan(),
                            m.version.dimmed(),
                            m.before_pos.to_string().red(),
                            m.after_pos.to_string().green()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn run_discover(args: DiscoverArgs) -> anyhow::Result<()> {
    let results = discover(&args.root, args.max_depth)
        .with_context(|| format!("discovering under {}", args.root.display()))?;

    if args.format == "json" {
        let s = serde_json::to_string(&results)?;
        println!("{s}");
        return Ok(());
    }

    let depth_label = match args.max_depth {
        Some(n) => format!("max depth: {n}"),
        None => "max depth: unlimited".to_string(),
    };
    println!(
        "{} {} ({})",
        "Discover".bold(),
        args.root.display(),
        depth_label.dimmed()
    );
    if results.is_empty() {
        println!("  (no olibs found)");
    } else {
        for d in &results {
            println!(
                "  {} {}  {}",
                "✓".green(),
                d.path.display(),
                format!("({} .md)", d.md_count).dimmed()
            );
        }
    }
    println!("{} olib(s) found.", results.len());
    Ok(())
}

fn run_inspect(args: InspectArgs, opts: ParseOptions) -> anyhow::Result<()> {
    let element = parse_file_with(&args.path, opts)
        .with_context(|| format!("reading {}", args.path.display()))?;

    let body_lines = if element.body.is_empty() {
        0
    } else {
        element.body.lines().count()
    };
    let body_chars = element.body.chars().count();

    if args.format == "json" {
        // Flatten header into the top-level JSON object and add
        // body summary fields. One line, easy for agents to pipe.
        let report = serde_json::json!({
            "name":         element.header.name,
            "kind":         element.header.kind,
            "id":           element.header.id,
            "version":      element.header.version,
            "meta":         element.header.meta,
            "generated_at": element.header.generated_at,
            "render_mode":  element.header.render_mode,
            "body_level":   element.header.body_level,
            "depth":        element.header.depth,
            "composed_of":  element.header.composed_of,
            "body_lines":   body_lines,
            "body_chars":   body_chars,
        });
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }

    // Human format
    println!("{} {}", "Inspect".bold(), args.path.display());
    println!("  {:9} {}", "id".dimmed(), element.header.id);
    println!("  {:9} {}", "name".dimmed(), element.header.name);
    println!("  {:9} {:?}", "kind".dimmed(), element.header.kind);
    println!("  {:9} {}", "version".dimmed(), element.header.version);
    if !element.header.meta.is_empty() {
        println!("  {:9} {}", "meta".dimmed(), element.header.meta);
    }
    if let Some(gen_at) = &element.header.generated_at {
        println!("  {:9} {}", "generated".dimmed(), gen_at);
    }
    if let Some(rm) = &element.header.render_mode {
        println!("  {:9} {}", "render".dimmed(), rm);
    }
    if let Some(bl) = element.header.body_level {
        println!("  {:9} {}", "body_lvl".dimmed(), bl);
    }
    if let Some(depth) = element.header.depth {
        println!("  {:9} {}", "depth".dimmed(), depth);
    }
    if let Some(inputs) = &element.header.composed_of {
        println!("  {:9} {} input(s)", "recipe".dimmed(), inputs.len());
        for i in inputs {
            println!("    - {} @ {}", i.id.cyan(), i.version.dimmed());
        }
    }
    println!(
        "  {:9} {} line(s), {} chars",
        "body".dimmed(),
        body_lines,
        body_chars
    );
    Ok(())
}

fn run_migrate(args: MigrateArgs) -> anyhow::Result<()> {
    eprintln!(
        "{} migrating {} (in-place; back up with git before running)",
        "WARNING:".yellow().bold(),
        args.library.display()
    );
    let summary = migrate_library(&args.library)?;
    println!(
        "{} {} migrated, {} skipped, {} failed",
        "Migrate".green().bold(),
        summary.migrated.len(),
        summary.skipped.len(),
        summary.failed.len()
    );
    for path in &summary.migrated {
        println!("  {} {}", "✓".green(), path.display());
    }
    for (path, reason) in &summary.skipped {
        println!("  {} {} ({})", "-".dimmed(), path.display(), reason);
    }
    for (path, err) in &summary.failed {
        eprintln!("  {} {}: {}", "✗".red(), path.display(), err);
    }
    if !summary.failed.is_empty() {
        anyhow::bail!("{} file(s) failed to migrate", summary.failed.len());
    }
    Ok(())
}

// Re-export to ensure unused-import warnings don't fire on `serialize` import path.
#[allow(dead_code)]
fn _unused_imports_silencer() {
    let _ = serialize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_md_extension_is_case_insensitive() {
        assert!(has_md_extension(Path::new("a.md")));
        assert!(has_md_extension(Path::new("a.MD")));
        assert!(has_md_extension(Path::new("dir/a.Md")));
        assert!(!has_md_extension(Path::new("a.txt")));
        assert!(!has_md_extension(Path::new("a")));
    }

    #[test]
    fn resolve_id_uses_kebab_stem_directly() {
        match resolve_id(Path::new("drafts/numbered-sprints.md"), false).unwrap() {
            IdResolution::Use { id, rename_to } => {
                assert_eq!(id, "numbered-sprints");
                assert!(rename_to.is_none());
            }
            IdResolution::Skip(_) => panic!("a kebab-case stem should resolve directly"),
        }
    }

    #[test]
    fn resolve_id_slugs_non_kebab_with_flag() {
        match resolve_id(Path::new("drafts/My Draft.md"), true).unwrap() {
            IdResolution::Use { id, rename_to } => {
                assert_eq!(id, "my-draft");
                assert!(rename_to.is_none());
            }
            IdResolution::Skip(_) => panic!("--slug should resolve a non-kebab stem"),
        }
    }

    #[test]
    fn resolve_id_skips_non_kebab_without_tty_or_flag() {
        // Under `cargo test` stdin is not a terminal, so a non-kebab stem
        // with no --slug must skip rather than block on a prompt.
        let r = resolve_id(Path::new("drafts/My Draft.md"), false).unwrap();
        assert!(matches!(r, IdResolution::Skip(_)));
    }
}
