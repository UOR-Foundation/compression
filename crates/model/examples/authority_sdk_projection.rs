//! Diagnostic explicit-reader harness; not an installed SDK fallback or a gate.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use repo_model::authority::ModuleReader;
use repo_model::{codegen, AuthorityGraph};
use serde_json::Value;

struct ExplicitSdkReader {
    executable: PathBuf,
}

impl ModuleReader for ExplicitSdkReader {
    fn read_module(&self, source: &str) -> Result<Value, String> {
        let mut child = Command::new(&self.executable)
            .arg("read")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                format!("explicit SDK reader {}: {error}", self.executable.display())
            })?;
        child
            .stdin
            .take()
            .ok_or("reader stdin is absent")?
            .write_all(source.as_bytes())
            .map_err(|error| error.to_string())?;
        let output = child
            .wait_with_output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "explicit SDK reader exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        serde_json::from_slice(&output.stdout)
            .map_err(|error| format!("reader returned invalid JSON: {error}"))
    }
}

fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 || !["--write", "--check"].contains(&args[3].to_string_lossy().as_ref()) {
        return Err("usage: authority_sdk_projection SOURCE_ROOT EXPLICIT_SDK_READER PROJECTION_ROOT --write|--check".into());
    }
    let root = Path::new(&args[0]);
    let output = Path::new(&args[2]);
    let reader = ExplicitSdkReader {
        executable: PathBuf::from(&args[1]),
    };
    let graph = AuthorityGraph::load_with_reader(root, &reader)?;
    graph.model().check()?;
    let projections = codegen::render_all(&graph);
    let mut paths = Vec::new();
    for (relative, expected) in &projections {
        let path = output.join(relative);
        if args[3] == "--write" {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, expected)?;
        } else if std::fs::read_to_string(&path)? != *expected {
            return Err(format!(
                "{} is stale: differs from actual SDK-read authority projection",
                path.display()
            )
            .into());
        }
        paths.push(relative);
    }
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "scope":"diagnostic explicit official reader; not pinned SDK acceptance",
            "source_root":root,"reader":reader.executable,"projection_root":output,
            "mode":args[3].to_string_lossy(),"paths":paths,
            "claims":graph.ids.id.len(),"varint_kernel_theorems":graph.varint.theorem.len()
        }))?
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    match execute() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("authority reader diagnostic failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
