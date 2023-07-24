#![allow(unused_imports)]
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crate::common::pool::ThreadPool;
use crate::{ printdb, printdbf, printerr };

/* const PORT: u16 = 8080;
const HOST: [u8;4] = [127, 0, 0, 1];
const POOL_SIZE: usize = 50; */

#[derive(Debug)]
pub struct WebServer {
    listener: TcpListener,
    pool: ThreadPool,
}

impl WebServer {
    pub fn new(port: u16, host: [u8;4], pool_size: usize) -> Self {
        let sock = SocketAddr::from((host, port));

        Self { 
            listener: TcpListener::bind(sock).unwrap(),
            pool: ThreadPool::new(pool_size)
        }
    }

    pub fn connect(&self) {
        for inc in self.listener.incoming() {
            match inc {
                Ok(stream) => {
                    // not certain about this 'move'
                    self.pool.execute(move ||
                        handle_connection(stream)
                    );
                }
                Err(e) => printerr!("Connection failed:", e)
            }
        }
    }
}

fn handle_connection(stream: TcpStream) {
    match BufReader::new(&stream).lines().next() {
        Some(header) => match header {
            Ok(header) => {
                match header.as_str() {
                    "GET / HTTP/1.1" => {
                        send_response(stream, "HTTP/1.1 200 OK", "index.html")
                    }
                    "GET /sleep HTTP/1.1" => {
                        /* for testing */
                        thread::sleep(Duration::from_secs(5));
                        send_response(stream, "HTTP/1.1 200 OK", "index.html")
                    }
                    _ => send_response(stream, "HTTP/1.1 200 OK", "index.html")
                }
            }
            Err(msg) => eprintln!("Error reading line: {}", msg)
        }
        None => eprintln!("failed to read request from stream")
    }
}

fn send_response(mut stream: TcpStream, http_status: &str, html_path: &str) {
    match fs::read_to_string(html_path) {
        Ok(txt) => {
            let response = format!(
                "{}\r\nContent-Length: {}\r\n\r\n{}",
                http_status,
                txt.len(),
                txt,
            );
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("error writing to TcpStream: {}", e);
            }
        }
        Err(e) => eprintln!("error reading from file '{}': {}", html_path, e)
    }
}

// # Response format:
// HTTP-Version Status-Code Reason-Phrase CRLF      ## HTTP/1.1 200 OK\r\n
// headers CRLF                                     ## <headers>\r\n
// message-body

