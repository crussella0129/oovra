//! `oovra` — CLI front-end.
//!
//! Four subcommands:
//!   - create    Scaffold a new atom or label an existing .md file
//!   - compose   Compose ordered inputs into a compound
//!   - decompose Recover a compound's inputs (one level, or --full)
//!   - compare   Diff two prompt elements (kind-aware)

use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use oovra::create::{label, scaffold, LabelArgs, ScaffoldArgs};
use oovra::decompose::{decompose_full, report};
use oovra::diff::{compare, DiffReport};
use oovra::element::{parse_file, serialize, write, PromptElement};
use oovra::library::Library;
use oovra::render::{compose, render_text, ComposeRequest};

#[derive(Parser, Debug)]
#[command(name = "oovra", version, about = "Compose and compare agentic prompt elements (Markdown + TOML).", long_about = None)]
struct Cli {
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
}

#[derive(clap::Args, Debug)]
struct CreateArgs {
    /// Scaffold a new atom with this ID
    #[arg(long, group = "mode")]
    new: Option<String>,

    /// Label an existing .md file at this path as an Oovra element
    #[arg(long, value_name = "PATH", group = "mode")]
    label: Option<PathBuf>,

    /// Override the ID. Defaults: for --new, the value of --new; for --label, the file stem.
    #[arg(long)]
    id: Option<String>,

    /// Human-readable name. Defaults to the ID.
    #[arg(long)]
    name: Option<String>,

    /// Semver version
    #[arg(long, default_value = "1.0.0")]
    version: String,

    /// Meta description
    #[arg(long, default_value = "")]
    meta: String,

    /// Library directory (where new elements are written)
    #[arg(long, default_value = "./elements")]
    library: PathBuf,

    /// Force-overwrite a file that already has an Oovra header (--label only)
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
    #[arg(long, default_value = "./elements")]
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Create(args) => run_create(args),
        Command::Compose(args) => run_compose(args),
        Command::Decompose(args) => run_decompose(args),
        Command::Compare(args) => run_compare(args),
    }
}

fn run_create(args: CreateArgs) -> anyhow::Result<()> {
    match (args.new.as_ref(), args.label.as_ref()) {
        (Some(new_id), None) => {
            let id = args.id.unwrap_or_else(|| new_id.clone());
            let path = scaffold(ScaffoldArgs {
                library_dir: args.library.clone(),
                id: id.clone(),
                name: args.name,
                version: args.version,
                meta: args.meta,
            })
            .with_context(|| format!("scaffolding element '{id}'"))?;
            println!(
                "{} {}",
                "Created".green().bold(),
                path.display()
            );
            Ok(())
        }
        (None, Some(source_path)) => {
            let id = args
                .id
                .or_else(|| {
                    source_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .ok_or_else(|| anyhow!("could not derive an ID from --label path; pass --id explicitly"))?;
            let path = label(LabelArgs {
                source_path: source_path.clone(),
                id: id.clone(),
                name: args.name,
                version: args.version,
                meta: args.meta,
                force: args.force,
            })
            .with_context(|| format!("labeling element '{id}'"))?;
            println!(
                "{} {}",
                "Labeled".green().bold(),
                path.display()
            );
            Ok(())
        }
        (Some(_), Some(_)) => Err(anyhow!("pass exactly one of --new or --label")),
        (None, None) => Err(anyhow!("pass exactly one of --new or --label")),
    }
}

fn run_compose(args: ComposeArgs) -> anyhow::Result<()> {
    let library = Library::load(&args.library)
        .with_context(|| format!("loading library from {}", args.library.display()))?;

    if let Some(re_render_path) = args.re_render {
        let existing = parse_file(&re_render_path)
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
        composed.header.composed_of.as_ref().map(|v| v.len()).unwrap_or(0)
    );
    Ok(())
}

fn run_decompose(args: DecomposeArgs) -> anyhow::Result<()> {
    let element = parse_file(&args.path)
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

fn run_compare(args: CompareArgs) -> anyhow::Result<()> {
    let a = parse_file(&args.a).with_context(|| format!("reading {}", args.a.display()))?;
    let b = parse_file(&args.b).with_context(|| format!("reading {}", args.b.display()))?;
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
                    for i in &s.added {
                        println!("    {} {}", "+".green(), format!("{} @ {}", i.id, i.version).green());
                    }
                }
                if !s.removed.is_empty() {
                    println!("  removed inputs:");
                    for i in &s.removed {
                        println!("    {} {}", "-".red(), format!("{} @ {}", i.id, i.version).red());
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
            }
        }
    }

    Ok(())
}

// Re-export to ensure unused-import warnings don't fire on `serialize` import path.
#[allow(dead_code)]
fn _unused_imports_silencer() {
    let _ = serialize;
}
