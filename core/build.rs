use std::error::Error;
use vergen::EmitBuilder;

use anyhow::{Result};
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks, Progress};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{self, BufRead, Write};
use std::fs;

// https://docs.rs/vergen/latest/vergen/struct.EmitBuilder.html#method.emit
fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo:rerun-if-changed=build.rs");

  let target = env::var("TARGET")?;
  let (goos, goarch) = match target.as_str() {
    t if t.contains("linux") => ("linux", if t.contains("aarch64") { "arm64" } else { "amd64" }),
    t if t.contains("windows") => ("windows", if t.contains("aarch64") { "arm64" } else { "amd64" }),
    t if t.contains("darwin") => ("darwin", if t.contains("aarch64") { "arm64" } else { "amd64" }),
    _ => {return Err(format!("Unsupported target: {}", target).into())},
  };

  let out_dir = env::var("OUT_DIR")?;
  let go_binary_name = if target.contains("windows") {
    "waglaylabridge.exe"
  } else {
    "waglaylabridge"
  };
  let go_binary_path = Path::new(&out_dir).join(go_binary_name);

  // Check if the Go binary already exists
  if !go_binary_path.exists() {
    // Clone the Git repository if it doesn't exist
    let repo_dir = Path::new(&out_dir).join("waglayla-stratum-bridge");
    if !repo_dir.exists() {
      println!("cargo:warning=Cloning waglayla-stratum-bridge");

      let mut fetch_opts = FetchOptions::new();
      let mut callbacks = RemoteCallbacks::new();
      callbacks.transfer_progress(|stats: Progress| {
          println!(
              "cargo:warning=Receiving objects: {} / {} ({} bytes)",
              stats.received_objects(),
              stats.total_objects(),
              stats.received_bytes()
          );
          true
      });

      fetch_opts.remote_callbacks(callbacks);
      fetch_opts.depth(1);
      fetch_opts.download_tags(git2::AutotagOption::All);

      let mut builder = RepoBuilder::new();
      builder.fetch_options(fetch_opts);

      let repo = builder.clone(
        "https://github.com/waglayla/waglayla-stratum-bridge.git",
        &repo_dir,
      )?;

      let tag_name = "v1.2.0";
      println!("cargo:warning=Checking out tag {} for waglayla-stratum-bridge", tag_name);
      let (object, reference) = repo.revparse_ext(tag_name)?;
      repo.checkout_tree(&object, None)?;
      match reference {
        Some(gref) => repo.set_head(gref.name().unwrap())?,
        None => repo.set_head_detached(object.id())?,
      }
    }

    // Check if Go is installed
    if !Command::new("go").arg("version").status()?.success() {
      return Err("Go toolchain not found. Please install Go.".into())
    }

    let main_dir = repo_dir.join("cmd").join("waglaylabridge");

    // Build the Go binary
    println!("cargo:warning=Building waglayla-stratum-bridge for {}", target.as_str());
      let mut child = Command::new("go")
        .current_dir(&repo_dir)
        .env("GOOS", goos)
        .env("GOARCH", goarch)
        .args(&["build", "-o", go_binary_path.to_str().unwrap(), main_dir.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_thread = std::thread::spawn(move || {
      let reader = io::BufReader::new(stdout);
      for line in reader.lines() {
        if let Ok(line) = line {
          println!("cargo:warning={}", line);
        }
      }
    });

    let stderr_thread = std::thread::spawn(move || {
      let reader = io::BufReader::new(stderr);
      for line in reader.lines() {
          if let Ok(line) = line {
            println!("cargo:warning={}", line);
          }
      }
    });

    let status = child.wait()?;
    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    if !status.success() {
        eprintln!("Go build failed with status: {}", status);
        std::process::exit(1);
    }
    println!("cargo:warning=Built waglayla-stratum-bridge");

    let embed_file_path = Path::new(&out_dir).join("embedded_executable.rs");
    let mut embed_file = fs::File::create(&embed_file_path).unwrap();
    writeln!(
      embed_file,
      "pub const BINARY: &[u8] = include_bytes!(r\"{}\");",
      go_binary_path.display()
    )
    .unwrap();

    println!("cargo:rerun-if-changed={}", go_binary_path.display());
    println!("cargo:rerun-if-changed={}", embed_file_path.display());
  }


  static_vcruntime::metabuild();
  EmitBuilder::builder()
    .all_build()
    .all_cargo()
    .all_git()
    .all_rustc()
    .emit()?;
  Ok(())
}
