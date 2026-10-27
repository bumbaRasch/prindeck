use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::escl::EsclClient;

#[derive(Debug)]
pub(super) enum Reply {
    Page(Vec<u8>),
    End,
    Canceled,
    Failure,
}

#[derive(Debug)]
pub(super) struct EsclServer {
    base: reqwest::Url,
    requests: Arc<Mutex<Vec<String>>>,
    stopped: Arc<AtomicBool>,
}

impl EsclServer {
    pub(super) fn spawn(replies: Vec<Reply>) -> EsclServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local listener");
        let addr = listener.local_addr().expect("read the local address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stopped = Arc::new(AtomicBool::new(false));

        {
            let requests = Arc::clone(&requests);
            let stopped = Arc::clone(&stopped);
            std::thread::spawn(move || serve(listener, replies, requests, stopped));
        }

        let base =
            reqwest::Url::parse(&format!("http://{addr}/eSCL/")).expect("build the eSCL base URL");

        EsclServer {
            base,
            requests,
            stopped,
        }
    }

    pub(super) fn client(&self) -> EsclClient {
        EsclClient::new(self.base.clone()).expect("build a test client")
    }

    pub(super) fn job_url(&self) -> reqwest::Url {
        self.base
            .join("ScanJobs/test-job")
            .expect("build the job URL")
    }

    pub(super) fn request_lines(&self) -> Vec<String> {
        self.requests
            .lock()
            .map(|lines| lines.clone())
            .unwrap_or_default()
    }
}

impl Drop for EsclServer {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
    }
}

fn serve(
    listener: TcpListener,
    replies: Vec<Reply>,
    requests: Arc<Mutex<Vec<String>>>,
    stopped: Arc<AtomicBool>,
) {
    listener
        .set_nonblocking(true)
        .expect("switch the listener to non-blocking mode");

    let mut pending: VecDeque<Reply> = replies.into();

    while !stopped.load(Ordering::Relaxed) {
        let (mut stream, _) = match listener.accept() {
            Ok(accepted) => accepted,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            Err(_) => break,
        };

        let Some(line) = read_request_line(&mut stream) else {
            continue;
        };

        if let Ok(mut lines) = requests.lock() {
            lines.push(line.clone());
        }

        let response = response_for(&line, &mut pending);
        let _ = stream.write_all(&response);
        let _ = stream.flush();
    }
}

fn read_request_line(stream: &mut TcpStream) -> Option<String> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];

    while !head.ends_with(b"\r\n\r\n") {
        match stream.read(&mut byte) {
            Ok(0) => return None,
            Ok(_) => head.push(byte[0]),
            Err(_) => return None,
        }
    }

    let text = String::from_utf8_lossy(&head);
    Some(text.lines().next().unwrap_or_default().to_string())
}

fn response_for(line: &str, pending: &mut VecDeque<Reply>) -> Vec<u8> {
    if line.starts_with("DELETE ") {
        return status_response(200, "OK", &[]);
    }

    match pending.pop_front() {
        Some(Reply::Page(bytes)) => status_response(200, "OK", &bytes),
        Some(Reply::End) | None => status_response(404, "Not Found", &[]),
        Some(Reply::Canceled) => status_response(410, "Gone", &[]),
        Some(Reply::Failure) => status_response(500, "Internal Server Error", &[]),
    }
}

fn status_response(status: u16, reason: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}
