use clap::{Args, Parser, Subcommand};
use ethos_model::{compare, Edit, Error, FieldEdit, Model, Result};
use serde_json::json;
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(
    name = "ethos-model",
    version,
    about = "Inspect, compare and repair ETHOS model files offline"
)]
struct Options {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Inspect {
        file: PathBuf,
    },
    /// Inspect all original bytes behind one decoded field or opaque region.
    Bytes {
        file: PathBuf,
        field: String,
    },
    Diff {
        before: PathBuf,
        after: PathBuf,
    },
    Preview {
        file: PathBuf,
        #[command(flatten)]
        edits: Changes,
    },
    Repair {
        file: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[command(flatten)]
        edits: Changes,
    },
    Validate {
        file: PathBuf,
        #[arg(long)]
        runtime: PathBuf,
        #[arg(long)]
        adapter: Option<PathBuf>,
    },
}
#[derive(Args)]
struct Changes {
    /// Decimal slot:hex Physical ID:hex Application ID. A dash preserves that ID.
    #[arg(long="set",value_name="SLOT:PHYS:APP",value_parser=parse_edit)]
    sensors: Vec<Edit>,
    /// Set a verified field by its stable inspection path.
    #[arg(long="field",value_name="PATH=VALUE",value_parser=parse_field)]
    fields: Vec<FieldEdit>,
}
fn parse_field(s: &str) -> std::result::Result<FieldEdit, String> {
    let (path, value) = s.split_once('=').ok_or("expected PATH=VALUE")?;
    if path.is_empty() {
        return Err("expected a field path".into());
    }
    Ok(FieldEdit {
        path: path.into(),
        value: value.into(),
    })
}
fn parse_edit(s: &str) -> std::result::Result<Edit, String> {
    let parts: Vec<_> = s.split(':').collect();
    let [slot, physical, app] = parts.as_slice() else {
        return Err("expected SLOT:PHYS:APP, e.g. 0:03:6802".into());
    };
    let slot = slot
        .parse::<usize>()
        .map_err(|_| "slot must be a zero-based decimal number")?;
    fn id(s: &str, width: usize) -> std::result::Result<Option<u16>, String> {
        if s == "-" {
            return Ok(None);
        }
        let s = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);
        if s.is_empty() || s.len() > width || !s.bytes().all(|v| v.is_ascii_hexdigit()) {
            return Err(format!("expected up to {width} hexadecimal digits or -"));
        }
        u16::from_str_radix(s, 16)
            .map(Some)
            .map_err(|e| e.to_string())
    }
    Ok(Edit {
        slot,
        physical_id: id(physical, 2)?.map(|v| v as u8),
        application_id: id(app, 4)?,
    })
}
fn run(opts: Options) -> Result<()> {
    let structured = opts.json;
    match opts.command {
        Command::Inspect { file } => {
            let m = Model::open(&file)?;
            if structured {
                println!(
                    "{}",
                    json!({"fields":m.fields(),"sensors":m.sensors(),"warnings":m.warnings()})
                );
            } else {
                for f in m.fields() {
                    println!(
                        "{}  {} = {}  [0x{:X}..0x{:X}){}",
                        f.section,
                        f.path,
                        f.value,
                        f.range.start,
                        f.range.end,
                        if f.editable { "  editable" } else { "" }
                    );
                }
                for warning in m.warnings() {
                    println!("Unverified: {warning}");
                }
            }
        }
        Command::Bytes { file, field } => {
            let m = Model::open(&file)?;
            let f = m
                .fields()
                .into_iter()
                .find(|f| f.path == field)
                .ok_or_else(|| Error::Invalid(format!("{field}: unknown field path")))?;
            let data = m
                .raw(f.range.clone())
                .expect("parsed field is in original bytes");
            if structured {
                println!("{}", json!({"path":f.path,"range":f.range,"bytes":data}));
            } else {
                for (n, line) in data.chunks(16).enumerate() {
                    println!(
                        "{:08X}  {}",
                        f.range.start + 16 * n,
                        line.iter()
                            .map(|v| format!("{v:02X}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
            }
        }
        Command::Diff { before, after } => {
            let l = Model::open(&before)?;
            let r = Model::open(&after)?;
            let ds = compare(&l, &r);
            if structured {
                println!(
                    "{}",
                    json!({"differences":ds,"before_warnings":l.warnings(),"after_warnings":r.warnings()})
                );
            } else {
                for warning in l.warnings().iter().chain(r.warnings()) {
                    println!("Unverified: {warning}");
                }
                if ds.is_empty() {
                    println!("No stored differences.");
                }
                for d in ds {
                    println!(
                        "{}  {}  {}  {} -> {}",
                        d.section,
                        d.kind,
                        d.path,
                        d.before.as_deref().unwrap_or("--"),
                        d.after.as_deref().unwrap_or("--")
                    );
                }
            }
        }
        Command::Preview { file, edits } => change(file, None, edits, structured)?,
        Command::Repair {
            file,
            output,
            edits,
        } => change(file, Some(output), edits, structured)?,
        Command::Validate {
            file,
            runtime,
            adapter,
        } => {
            let report = ethos_model_validation::validate_in_ethos(
                &file,
                &runtime,
                &adapter.unwrap_or_else(ethos_model_validation::default_adapter),
            )?;
            if structured {
                println!("{}", json!(report));
            } else {
                print!("{report}");
            }
        }
    }
    Ok(())
}
fn change(file: PathBuf, output: Option<PathBuf>, edits: Changes, structured: bool) -> Result<()> {
    if edits.sensors.is_empty() && edits.fields.is_empty() {
        return Err(Error::Invalid(
            "supply at least one --set or --field".into(),
        ));
    }
    let m = Model::open(&file)?;
    let changes = m.preview_fields(&edits.sensors, &edits.fields)?;
    if let Some(p) = &output {
        m.save_fields_copy(p, &edits.sensors, &edits.fields)?;
    }
    if structured {
        println!("{}", json!({"output":output,"changes":changes}));
    } else {
        for e in &changes {
            println!("{}  {} -> {}", e.path, e.before, e.after);
        }
        if changes.is_empty() {
            println!("No changes.");
        }
        if let Some(p) = &output {
            println!("Verified copy: {}", p.display());
        }
    }
    Ok(())
}
fn main() -> ExitCode {
    let opts = Options::parse();
    let json = opts.json;
    match run(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if json {
                eprintln!("{}", json!({"error":e.to_string()}));
            } else {
                eprintln!("{e}");
            }
            ExitCode::FAILURE
        }
    }
}
