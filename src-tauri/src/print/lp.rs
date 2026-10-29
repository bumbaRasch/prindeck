use std::fmt;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::{CupsJob, PrintOptions};

const TIMEOUT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub enum PrintError {
    Io {
        program: String,
        source: io::Error,
    },
    Timeout {
        program: String,
    },
    Failed {
        program: String,
        status: ExitStatus,
        stderr: String,
    },
    UnexpectedOutput {
        program: String,
        stdout: String,
    },
    MissingJobId,
}

impl PrintError {
    fn io(program: &str, source: io::Error) -> PrintError {
        PrintError::Io {
            program: program.to_string(),
            source,
        }
    }
}

impl fmt::Display for PrintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrintError::Io { program, source } => write!(f, "failed to run {program}: {source}"),
            PrintError::Timeout { program } => {
                write!(f, "{program} did not finish within the timeout")
            }
            PrintError::Failed {
                program,
                status,
                stderr,
            } => write!(f, "{program} failed with {status}: {stderr}"),
            PrintError::UnexpectedOutput { program, stdout } => {
                write!(f, "{program} printed no job id: {stdout}")
            }
            PrintError::MissingJobId => write!(f, "the job id is not set"),
        }
    }
}

impl std::error::Error for PrintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PrintError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub fn print_file(queue: &str, path: &Path, opts: &PrintOptions) -> Result<String, PrintError> {
    let copies = opts.copies.to_string();
    let sides = format!("sides={}", opts.sides.as_lp_option());
    let file = path.to_string_lossy();
    let stdout = run("lp", &["-d", queue, "-n", &copies, "-o", &sides, &file])?;

    job_id_from_lp_output(&stdout).ok_or_else(|| PrintError::UnexpectedOutput {
        program: "lp".to_string(),
        stdout,
    })
}

pub fn queue_jobs(queue: &str) -> Result<Vec<CupsJob>, PrintError> {
    let stdout = run("lpstat", &["-W", "not-completed", "-o", queue])?;
    Ok(parse_jobs(queue, &stdout))
}

pub fn cancel(job_id: &str) -> Result<(), PrintError> {
    let job_id = job_id.trim();

    if job_id.is_empty() {
        return Err(PrintError::MissingJobId);
    }

    run("cancel", &[job_id])?;
    Ok(())
}

fn run(program: &str, args: &[&str]) -> Result<String, PrintError> {
    let mut child = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| PrintError::io(program, error))?;

    let deadline = Instant::now() + TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(PrintError::Timeout {
                        program: program.to_string(),
                    });
                }
                thread::sleep(POLL_INTERVAL);
            }
            Err(error) => return Err(PrintError::io(program, error)),
        }
    };

    let stdout = read_pipe(program, child.stdout.take())?;
    let stderr = read_pipe(program, child.stderr.take())?;

    if !status.success() {
        return Err(PrintError::Failed {
            program: program.to_string(),
            status,
            stderr: stderr.trim().to_string(),
        });
    }

    Ok(stdout)
}

fn read_pipe<R: Read>(program: &str, pipe: Option<R>) -> Result<String, PrintError> {
    let Some(mut pipe) = pipe else {
        return Ok(String::new());
    };

    let mut buffer = String::new();
    pipe.read_to_string(&mut buffer)
        .map_err(|error| PrintError::io(program, error))?;
    Ok(buffer)
}

pub(super) fn parse_jobs(queue: &str, output: &str) -> Vec<CupsJob> {
    let prefix = format!("{queue}-");

    output
        .lines()
        .filter_map(|line| {
            let mut tokens = line.split_whitespace();
            let id = tokens.next()?;
            if !id.starts_with(&prefix) {
                return None;
            }

            let owner = tokens.next()?;
            let size_kb = tokens.next().and_then(|size| size.parse::<u64>().ok())?;
            Some(CupsJob {
                id: id.to_string(),
                owner: owner.to_string(),
                size_kb,
            })
        })
        .collect()
}

fn job_id_from_lp_output(stdout: &str) -> Option<String> {
    stdout.lines().find_map(|line| {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let position = tokens
            .windows(3)
            .position(|window| window == ["request", "id", "is"])?;
        tokens.get(position + 3).map(|token| token.to_string())
    })
}
