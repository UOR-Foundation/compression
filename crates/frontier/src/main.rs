//! `frontier`: the claim register's command line.
//!
//! ```text
//! frontier check    [--register frontier.toml]
//! frontier pin      NAME --source SRC [--public] (--dir DIR | --git REV PATH...)
//! frontier compare  NAME [--dir DIR] [--source TEXT]
//! frontier decide   [--corpus NAME[=DIR]]... [--point KAPPA] [--label TEXT] [--by WHO] [--time]
//! frontier report   [--point KAPPA] --out DIR
//! ```
//!
//! Every command reads and writes `--register` (default `frontier.toml`) and
//! takes `--repo` (default `.`) for git sources and the export identity.
//! `decide` with no `--corpus` decides every corpus whose bytes it can reach
//! offline (git sources); with no `--point` it decides the shipped client's
//! point. `--by` is `local` unless the factory names itself
//! (`factory:<identity>`). `decide` also records the distance of the run
//! to each bound of `Uorc.Performance` (requirement claims); with `--time` it
//! times the run on this host too, named by its κ.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use frontier::{
    export_identity, CodecPoint, Comparator, Corpus, Error, Form, Host, Member, Register, Status,
    FAMILIES,
};

struct Args {
    command: String,
    positional: Vec<String>,
    options: BTreeMap<String, Vec<String>>,
    flags: Vec<String>,
}

fn parse() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let command = it.next().ok_or_else(|| "missing command".to_owned())?;
    let mut args = Args {
        command,
        positional: Vec::new(),
        options: BTreeMap::new(),
        flags: Vec::new(),
    };
    while let Some(a) = it.next() {
        match a.strip_prefix("--") {
            Some(flag @ ("public" | "time")) => args.flags.push(flag.to_owned()),
            Some(key) => {
                let v = it.next().ok_or_else(|| format!("--{key} needs a value"))?;
                args.options.entry(key.to_owned()).or_default().push(v);
            }
            None => args.positional.push(a),
        }
    }
    Ok(args)
}

impl Args {
    fn one(&self, key: &str) -> Option<&str> {
        self.options
            .get(key)
            .and_then(|v| v.last())
            .map(String::as_str)
    }
    fn all(&self, key: &str) -> &[String] {
        self.options.get(key).map_or(&[], Vec::as_slice)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("frontier: {e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse()?;
    let register_path = PathBuf::from(args.one("register").unwrap_or("frontier.toml"));
    let repo = PathBuf::from(args.one("repo").unwrap_or("."));
    let load = || -> Result<Register, String> {
        if register_path.exists() {
            Register::load(&register_path).map_err(|e| e.to_string())
        } else {
            Ok(Register::default())
        }
    };
    let save = |r: &Register| r.save(&register_path).map_err(|e| e.to_string());
    match args.command.as_str() {
        "check" => {
            let r = load()?;
            let text = std::fs::read_to_string(&register_path).map_err(|e| e.to_string())?;
            if r.to_text().map_err(|e| e.to_string())? != text {
                return Err(
                    "frontier.toml is not in canonical form (run any writing command)".into(),
                );
            }
            println!(
                "ok: {} corpora, {} points, {} claims",
                r.corpus.len(),
                r.point.len(),
                r.claim.len()
            );
            Ok(())
        }
        "pin" => {
            let mut r = load()?;
            let name = args.positional.first().ok_or("pin NAME")?;
            let source = args.one("source").ok_or("pin needs --source")?;
            let members = match (args.one("dir"), args.one("git")) {
                (Some(dir), None) => dir_members(Path::new(dir))?,
                (None, Some(rev)) => args.positional[1..]
                    .iter()
                    .map(|p| git_member(&repo, rev, p))
                    .collect::<Result<_, String>>()?,
                _ => return Err("pin needs --dir DIR or --git REV PATH...".into()),
            };
            let corpus = Corpus::new(
                name,
                source,
                args.flags.iter().any(|f| f == "public"),
                members,
            );
            println!(
                "{} {} ({} members, {} bytes)",
                corpus.name,
                corpus.kappa,
                corpus.members.len(),
                corpus.bytes()
            );
            r.add_corpus(corpus).map_err(|e| e.to_string())?;
            save(&r)
        }
        "compare" => {
            let mut r = load()?;
            let name = args.positional.first().ok_or("compare NAME")?;
            let corpus = r.corpus_named(name).map_err(|e| e.to_string())?.clone();
            let files = corpus
                .materialize(args.one("dir").map(Path::new), &repo)
                .map_err(|e| e.to_string())?;
            let source = args.one("source").unwrap_or("measured by frontier compare");
            for (tool, cmd, argv, version) in [
                ("gzip -9", "gzip", &["-9", "-c"][..], &["--version"][..]),
                ("bzip2 -9", "bzip2", &["-9", "-c"][..], &["--help"][..]),
                ("xz -9e", "xz", &["-9e", "-c"][..], &["--version"][..]),
            ] {
                let Some(version) = first_line(cmd, version) else {
                    continue;
                };
                let mut total = 0u64;
                for f in &files {
                    total += stdin_size(cmd, argv, f)?;
                }
                r.comparator
                    .retain(|c| !(c.corpus == corpus.kappa && c.tool == tool));
                r.comparator.push(Comparator {
                    corpus: corpus.kappa.clone(),
                    tool: tool.to_owned(),
                    version,
                    bytes: total,
                    source: source.to_owned(),
                });
                println!("{tool}: {total}");
            }
            save(&r)
        }
        "decide" => {
            let mut r = load()?;
            let by = args.one("by").unwrap_or("local").to_owned();
            let point = point_of(&r, &args, &repo)?;
            r.add_point(&point, args.one("label"));
            let targets: Vec<(Corpus, Option<PathBuf>)> = if args.all("corpus").is_empty() {
                r.corpus
                    .iter()
                    .filter(|c| c.source.starts_with("git:"))
                    .map(|c| (c.clone(), None))
                    .collect()
            } else {
                args.all("corpus")
                    .iter()
                    .map(|spec| {
                        let (name, dir) = match spec.split_once('=') {
                            Some((n, d)) => (n, Some(PathBuf::from(d))),
                            None => (spec.as_str(), None),
                        };
                        Ok((
                            r.corpus_named(name).map_err(|e| e.to_string())?.clone(),
                            dir,
                        ))
                    })
                    .collect::<Result<_, String>>()?
            };
            let mut refuted = 0;
            let threads = uorc::default_threads() as u64;
            let host = args.flags.iter().any(|f| f == "time").then(Host::this);
            for (corpus, dir) in targets {
                let q = Register::closed_question(&point, &corpus);
                let settled = r.settled(&point, &corpus, &by).is_some();
                let timed = host.as_ref().is_some_and(|h| {
                    !r.requirements_settled(&point, &corpus, threads, Some(h), &by)
                });
                let needs_run = !settled
                    || timed
                    || !r.requirements_settled(&point, &corpus, threads, None, &by);
                if needs_run {
                    let files = corpus
                        .materialize(dir.as_deref(), &repo)
                        .map_err(|e| e.to_string())?;
                    let run =
                        frontier::run(&point, &corpus, &files, timed).map_err(|e| e.to_string())?;
                    r.decide_run(&point, &corpus, &run, &by)
                        .map_err(|e| e.to_string())?;
                    let measured = host.as_ref().filter(|_| timed);
                    for c in r.decide_requirements(&point, &corpus, &run, threads, measured, &by) {
                        println!(
                            "  {} on {}: {} {} observed, bound {}, distance {}: {}",
                            c.requirement.as_deref().unwrap_or(""),
                            corpus.name,
                            c.observed.unwrap_or(0),
                            c.unit.as_deref().unwrap_or(""),
                            c.derived.unwrap_or(0),
                            c.distance.unwrap_or(0),
                            c.status.name()
                        );
                        refuted += usize::from(c.status == Status::Refuted);
                    }
                }
                let claim = r.known(&q.id).cloned().ok_or("decided claim")?;
                println!(
                    "closed {} on {}: {} bytes, {} steps: {}{}",
                    &point.kappa()[..12],
                    corpus.name,
                    claim.observed_bytes.or(claim.bytes).unwrap_or(0),
                    claim.observed_steps.or(claim.steps).unwrap_or(0),
                    claim.status.name(),
                    if settled { " (known by κ)" } else { "" }
                );
                refuted += usize::from(claim.status == Status::Refuted);
                for b in r.decide_bounds(&point, &corpus, &by) {
                    println!(
                        "  bound {} <= {} ({}): {}",
                        corpus.name,
                        b.bound.unwrap_or(0),
                        b.against.as_deref().unwrap_or(""),
                        b.status.name()
                    );
                }
                save(&r)?;
            }
            if refuted > 0 {
                return Err(format!(
                    "{refuted} claim(s) refuted: the codec did not reproduce them"
                ));
            }
            Ok(())
        }
        "report" => {
            let r = load()?;
            let point = point_of(&r, &args, &repo)?;
            let out = PathBuf::from(args.one("out").ok_or("report needs --out DIR")?);
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            std::fs::write(out.join("frontier.md"), report_md(&r, &point))
                .map_err(|e| e.to_string())?;
            std::fs::write(
                out.join("frontier.toml"),
                r.to_text().map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            println!("wrote {}/frontier.{{md,toml}}", out.display());
            Ok(())
        }
        other => Err(format!("unknown command {other}")),
    }
}

fn point_of(r: &Register, args: &Args, repo: &Path) -> Result<CodecPoint, String> {
    match args.one("point") {
        Some(k) => r
            .point
            .iter()
            .find(|p| p.kappa.starts_with(k))
            .ok_or_else(|| format!("no point {k}"))?
            .point()
            .map_err(|e| e.to_string()),
        None => Ok(CodecPoint::shipped(
            &export_identity(repo).map_err(|e: Error| e.to_string())?,
        )),
    }
}

fn dir_members(dir: &Path) -> Result<Vec<Member>, String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
        .iter()
        .map(|n| {
            let bytes = std::fs::read(dir.join(n)).map_err(|e| format!("{n}: {e}"))?;
            Ok(Member::of(n, &bytes))
        })
        .collect()
}

fn git_member(repo: &Path, rev: &str, path: &str) -> Result<Member, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["cat-file", "blob", &format!("{rev}:{path}")])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("git {rev}:{path} not found"));
    }
    Ok(Member::of(path, &out.stdout))
}

fn first_line(cmd: &str, argv: &[&str]) -> Option<String> {
    let out = std::process::Command::new(cmd).args(argv).output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_owned())
}

fn stdin_size(cmd: &str, argv: &[&str], input: &[u8]) -> Result<u64, String> {
    use std::io::Write;
    let mut child = std::process::Command::new(cmd)
        .args(argv)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("stdin")?;
    let data = input.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&data));
    let out = child
        .wait_with_output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    writer
        .join()
        .map_err(|_| "writer".to_owned())?
        .map_err(|e| format!("{cmd}: {e}"))?;
    if !out.status.success() {
        return Err(format!("{cmd} failed"));
    }
    Ok(out.stdout.len() as u64)
}

/// The release notes section: every claim about `point`, then the register's
/// other decided claims by form.
fn report_md(r: &Register, point: &CodecPoint) -> String {
    let k = point.kappa();
    let name = |kappa: &str| {
        r.corpus_by_kappa(kappa)
            .map_or("?".to_owned(), |c| c.name.clone())
    };
    let mut md = vec![
        "## Closed claims".to_owned(),
        String::new(),
        format!(
            "Codec point `{}` (export `{}`, plan {}, {} lane). Sizes and decode steps are exact: each was decided by running this codec over the κ-named corpus and is recorded with its decision in the release's `frontier.toml`. A step is one model input evaluated for one coded bit.",
            &k[..16],
            &point.export[..16.min(point.export.len())],
            point.plan_text().split(' ').next().unwrap_or(""),
            point.lanes
        ),
        String::new(),
        "| corpus | corpus κ | input bytes | archive bytes | decode steps | steps per byte | status | decided by |".to_owned(),
        "| --- | --- | ---: | ---: | ---: | ---: | --- | --- |".to_owned(),
    ];
    for c in r
        .claim
        .iter()
        .filter(|c| c.form == Form::Closed && c.codec.as_deref() == Some(&k))
    {
        let input = r.corpus_by_kappa(&c.corpus).map_or(0, Corpus::bytes);
        let steps = c.observed_steps.or(c.steps).unwrap_or(0);
        md.push(format!(
            "| {} | `{}` | {} | {} | {} | {:.2} | {} | {} |",
            name(&c.corpus),
            &c.corpus[..12],
            input,
            c.observed_bytes
                .or(c.bytes)
                .map_or("-".into(), |b| b.to_string()),
            steps,
            if input > 0 {
                steps as f64 / input as f64
            } else {
                0.0
            },
            c.status.name(),
            c.receipt
                .as_deref()
                .map_or("-", |b| b.split_once(':').map_or(b, |(w, _)| w)),
        ));
    }
    md.push(String::new());
    let bounds: Vec<_> = r
        .claim
        .iter()
        .filter(|c| c.form == Form::Bound && c.codec.as_deref() == Some(&k))
        .collect();
    if !bounds.is_empty() {
        md.push("## UC-PERF-01 target".to_owned());
        md.push(String::new());
        md.push("On every public corpus, the archive is no larger than each pinned comparator's. A refuted row is a comparator this codec does not yet reach.".to_owned());
        md.push(String::new());
        md.push("| corpus | comparator | comparator bytes | uorc bytes | status |".to_owned());
        md.push("| --- | --- | ---: | ---: | --- |".to_owned());
        for b in bounds {
            let own = r
                .known(
                    &Register::closed_question(
                        point,
                        r.corpus_by_kappa(&b.corpus).expect("known corpus"),
                    )
                    .id,
                )
                .and_then(|c| c.bytes)
                .unwrap_or(0);
            md.push(format!(
                "| {} | {} | {} | {} | {} |",
                name(&b.corpus),
                b.against.as_deref().unwrap_or(""),
                b.bound.unwrap_or(0),
                own,
                b.status.name()
            ));
        }
        md.push(String::new());
    }
    let requirements: Vec<_> = r
        .claim
        .iter()
        .filter(|c| c.form == Form::Requirement && c.codec.as_deref() == Some(&k))
        .collect();
    if !requirements.is_empty() {
        md.push("## Distance to each requirement".to_owned());
        md.push(String::new());
        md.push("Each requirement of `Uorc.Performance` derives a bound for this codec on each corpus; the row is what the run observed, the bound, and how far the observation is from it on the side the requirement forbids. A distance of 0 is the requirement met. Bytes and steps are exact and decided; rates are measured on the host named by its κ. `leastArchive`: the archive against the information the chosen model assigns to the input (what is left is the archive's form and the coder's rounding). `wholeHost`: per input, the longest chain of dependent decode steps against the total divided by the threads. `hostCeiling`: throughput against the input over the host's ceiling (`Uorc.Host::ceilingNanos`): the slower of the steps the run took (decode: the archive's exact steps; compress: the model steps every trial coded, `uorc::ncm::model_steps`) over the host's threads at the per-core step rate of the codec's own mixer contraction and the cache lines its tables move over the host's memory rate. The compute term covers the contraction only, so the distance includes the rest of each step; both sides count the archive's moved lines, a floor on what the compress trials move.".to_owned());
        md.push(String::new());
        md.push("| requirement | quantity | corpus | at | observed | bound | distance | unit | status |".to_owned());
        md.push("| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |".to_owned());
        for c in &requirements {
            let at = match (c.threads, &c.host) {
                (Some(t), _) => match c.met_up_to {
                    Some(m) => format!("{t} threads (met up to {m})"),
                    None => format!("{t} threads (met at any)"),
                },
                (None, Some(h)) => {
                    let host = r.host.iter().find(|x| &x.kappa == h);
                    format!(
                        "host `{}` ({})",
                        &h[..12],
                        host.map_or("?".to_owned(), |x| format!(
                            "{}, {} threads, {}",
                            x.cpu, x.threads, x.vector
                        ))
                    )
                }
                _ => "-".to_owned(),
            };
            md.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                c.requirement.as_deref().unwrap_or(""),
                c.quantity.as_deref().unwrap_or(""),
                name(&c.corpus),
                at,
                c.observed.map_or("-".into(), |v| v.to_string()),
                c.derived.map_or("-".into(), |v| v.to_string()),
                c.distance.map_or("-".into(), |v| v.to_string()),
                c.unit.as_deref().unwrap_or(""),
                c.status.name()
            ));
        }
        md.push(String::new());
    }
    let closed: Vec<_> = r
        .claim
        .iter()
        .filter(|c| {
            c.form == Form::Closed && c.codec.as_deref() == Some(&k) && c.status == Status::Proved
        })
        .collect();
    if let Some(c) = closed.first() {
        md.push("<details><summary>Per family (first claim's corpus)</summary>".to_owned());
        md.push(String::new());
        md.push("| family | members | archive bytes | decode steps |".to_owned());
        md.push("| --- | ---: | ---: | ---: |".to_owned());
        for f in FAMILIES {
            let ms: Vec<_> = c.members.iter().filter(|m| m.family == f).collect();
            if !ms.is_empty() {
                md.push(format!(
                    "| {f} | {} | {} | {} |",
                    ms.len(),
                    ms.iter().map(|m| m.bytes).sum::<u64>(),
                    ms.iter().map(|m| m.steps).sum::<u64>()
                ));
            }
        }
        md.push(String::new());
        md.push("</details>".to_owned());
        md.push(String::new());
    }
    let others = r
        .claim
        .iter()
        .filter(|c| c.status != Status::Minted && c.codec.as_deref() != Some(&k))
        .count();
    md.push(format!(
        "The register holds {} claims over {} corpora and {} codec points; {} concern other points (earlier releases, engine candidates) and are kept by κ, never rerun.",
        r.claim.len(),
        r.corpus.len(),
        r.point.len(),
        others
    ));
    md.push(String::new());
    md.join("\n")
}
