#[macro_use]
extern crate tracing;

use std::env::var;
use std::fs;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

fn main() {
    let worker_index: usize = var("WORKER_INDEX")
        .map(|s| s.parse().unwrap())
        .expect("WORKER_INDEX not set");
    let output_path: PathBuf = var("OUTPUT_PATH")
        .map(PathBuf::from)
        .expect("OUTPUT_PATH not set");
    let runner_log_path = output_path.join("worker-log");
    let runner_err_path = output_path.join("worker-err");
    fs::create_dir_all(runner_log_path.as_path()).expect("cannot create output dir");
    fs::create_dir_all(runner_err_path.as_path()).expect("cannot create output dir");

    let appender = tracing_appender::rolling::never(
        output_path.as_path(),
        format!("worker-{}.log", worker_index),
    );
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::builder().from_env_lossy())
        .with_writer(appender)
        .with_ansi(false)
        .init();
    info!("output path: {}", output_path.display());

    let runner_path: PathBuf = var("RUNNER_PATH")
        .map(PathBuf::from)
        .map_err(|e| {
            error!("RUNNER_PATH not set: {:?}", e);
            e
        })
        .unwrap();
    if !runner_path.is_file() {
        error!("runner binary not found at: {}", runner_path.display());
        std::process::exit(1);
    }
    let traces_path: PathBuf = var("TRACES_PATH")
        .map(PathBuf::from)
        .map_err(|e| {
            error!("TRACES_PATH not set: {:?}", e);
            e
        })
        .unwrap();
    info!("loading traces from: {}", traces_path.display());

    let total_workers: usize = var("TOTAL_WORKERS")
        .map(|s| s.parse().unwrap())
        .map_err(|e| {
            error!("TOTAL_WORKERS not set: {:?}", e);
            e
        })
        .unwrap();
    info!("total workers: {total_workers}, worker index: {worker_index}");

    let all_jobs = load_jobs(traces_path.as_path());
    info!("total jobs: {}", all_jobs.len());
    let worker_jobs = get_jobs(all_jobs, worker_index, total_workers);
    let job_count = worker_jobs.len();
    info!("loaded {job_count} jobs to run");

    for (idx, (origin_idx, job)) in worker_jobs.into_iter().enumerate() {
        let trace_name = job
            .file_stem()
            .expect("cannot get file name")
            .to_string_lossy();
        info!(
            "running job #{origin_idx} {current}/{job_count}: {trace_name}",
            current = idx + 1
        );
        let task = std::process::Command::new(runner_path.as_path())
            .arg(job.as_path())
            .arg("--output")
            .arg(output_path.as_path())
            .output();
        match task {
            Err(e) => {
                error!("failed to exec task {trace_name}: {:?}", e);
            }
            Ok(output) => {
                // write log
                let log_path = runner_log_path.join(format!("{}.log", trace_name));
                if !output.stdout.is_empty() {
                    fs::write(log_path.as_path(), output.stdout).ok();
                }
                // write error
                let err_path = runner_err_path.join(format!("{}.err", trace_name));
                if !output.stderr.is_empty() {
                    fs::write(err_path.as_path(), output.stderr).ok();
                }
                if !output.status.success() {
                    error!("task {trace_name} failed");
                } else {
                    info!("task {trace_name} finished");
                }
            }
        }
    }
}

fn load_jobs(traces_path: &Path) -> Vec<PathBuf> {
    let mut jobs = Vec::new();
    for entry in fs::read_dir(traces_path).expect("could not read traces") {
        let entry = entry.expect("could not resolve entry");
        let path = entry.path();
        if path.is_dir() {
            jobs.extend(load_jobs(path.as_path()));
        } else if path.is_file() && path.extension().unwrap() == "json" {
            jobs.push(path);
        }
    }
    jobs.sort();
    jobs
}

// balance jobs to workers
fn get_jobs(
    all_jobs: Vec<PathBuf>,
    worker_index: usize,
    total_workers: usize,
) -> Vec<(usize, PathBuf)> {
    all_jobs
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| idx % total_workers == worker_index)
        .collect()
}
