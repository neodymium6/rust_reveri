use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use crate::arena::error::{NetworkArenaServerError, ClientManagerError, NetworkArenaClientError};
use crate::board::core::Turn;

const SUPER_COMMAND_MARKER: &str = "##SUPER##";
const READ_TIMEOUT : std::time::Duration = std::time::Duration::from_secs(5);
const BUF_SIZE: usize = 1024;

#[derive(Debug)]
struct ClientManager {
    clients: [Option<TcpStream>; 2],
}

impl ClientManager {
    fn new() -> Self {
        ClientManager {
            clients: [None, None],
        }
    }

    fn add_client(&mut self, stream: TcpStream) -> Result<(), ClientManagerError> {
        for i in 0..2 {
            if self.clients[i].is_none() {
                stream.set_read_timeout(Some(READ_TIMEOUT))
                    .map_err(|e| ClientManagerError::from(e))?;
                self.clients[i] = Some(stream);
                return Ok(());
            }
        }
        Err(ClientManagerError::NoMoreClients)
    }

    fn is_full(&self) -> bool {
        self.clients.iter().all(|x| x.is_some())
    }

    fn is_ready(&mut self) -> Result<bool, ClientManagerError> {
        for mut stream in self.clients.iter_mut() {
            match stream.as_mut() {
                Some(stream) => {
                    stream.write_all(SUPER_COMMAND_MARKER.as_bytes())
                        .map_err(|e| ClientManagerError::from(e))?;
                    stream.write_all(b" isready\n")
                        .map_err(|e| ClientManagerError::from(e))?;
                    stream.flush()
                        .map_err(|e| ClientManagerError::from(e))?;
                    let mut buffer = [0; BUF_SIZE];
                    let mut response = String::new();
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        }
                        Err(e) => return Err(ClientManagerError::from(e)),
                    }
                    if response.trim() != "readyok" {
                        return Ok(false);
                    }
                }
                None => return Err(ClientManagerError::ClientNotExists),
            }
        }
        Ok(true)
    }
}

#[derive(Debug)]
pub struct NetworkArenaServer {
    game_per_iter: usize,
    client_manager: ClientManager,
    stats: (usize, usize, usize),
    pieces: (usize, usize),
}

impl NetworkArenaServer {
    pub fn new(game_per_iter: usize) -> Self {
        NetworkArenaServer {
            game_per_iter,
            client_manager: ClientManager::new(),
            stats: (0, 0, 0),
            pieces: (0, 0),
        }
    }

    pub fn start(&mut self, port: u16) -> Result<(), NetworkArenaServerError> {
        let listener = TcpListener::bind(format!("localhost:{}", port))?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    match self.client_manager.add_client(stream) {
                        Ok(_) => {
                            if self.client_manager.is_full() {
                                self.play()?;
                                self.stats = (0, 0, 0);
                                self.pieces = (0, 0);
                            }
                        }
                        Err(e) => {
                            return Err(NetworkArenaServerError::from(e));
                        }
                    }
                }
                Err(e) => {
                    return Err(NetworkArenaServerError::from(e));
                }
            }
        }
        Ok(())
    }

    fn play(&mut self) -> Result<(), NetworkArenaServerError> {
        if !self.client_manager.is_ready()
            .map_err(|e| NetworkArenaServerError::from(e))? {
            return Err(NetworkArenaServerError::ClientNotReady);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct NetworkArenaClient {
    command: Vec<String>,
    stats: (usize, usize, usize),
    pieces: (usize, usize),
}

impl NetworkArenaClient {
    pub fn new(command: Vec<String>) -> Self {
        NetworkArenaClient {
            command,
            stats: (0, 0, 0),
            pieces: (0, 0),
        }
    }

    fn start_process(command: &[String], turn: Turn) -> Result<(Child, ChildStdin, BufReader<ChildStdout>), std::io::Error> {
        let mut cmd = Command::new(&command[0]);
        for arg in command.iter().skip(1) {
            cmd.arg(arg);
        }
    
        match turn {
            Turn::Black => cmd.arg("BLACK"),
            Turn::White => cmd.arg("WHITE"),
        };
    
        let mut process = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
    
        let mut stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
    
        // ping-pong test
        writeln!(stdin, "ping")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Write error"))?;
        stdin.flush()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Flush error"))?;
    
        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        reader.read_line(&mut response)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Read error"))?;
        
        if response.trim() != "pong" {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid response"));
        }
    
        Ok((process, stdin, reader))
    }

    pub fn join(&mut self, port: u16) -> Result<(), NetworkArenaClientError> {
        let mut stream = TcpStream::connect(format!("localhost:{}", port))
            .map_err(|e| NetworkArenaClientError::from(e))?;
        stream.set_read_timeout(Some(READ_TIMEOUT))
            .map_err(|e| NetworkArenaClientError::from(e))?;

        let (mut process_b, mut stdin_b, mut reader_b) = NetworkArenaClient::start_process(&self.command, Turn::Black)
            .map_err(|e| NetworkArenaClientError::from(e))?;
        let (mut process_w, mut stdin_w, mut reader_w) = NetworkArenaClient::start_process(&self.command, Turn::White)
            .map_err(|e| NetworkArenaClientError::from(e))?;

        let mut buffer = [0; BUF_SIZE];
        let mut response = String::new();

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => return Err(NetworkArenaClientError::ConnectionBroken),
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    if response.ends_with("\n") {
                        if response.starts_with(SUPER_COMMAND_MARKER) {
                            let command_line = response.trim_start_matches(SUPER_COMMAND_MARKER).trim();
                            let command: Vec<&str> = command_line.split_whitespace().collect();
                            match command[0] {
                                "isready" => {
                                    stream.write_all(b"readyok\n")
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stream.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                },
                                "black" => {
                                    stdin_b.write_all(command[1].as_bytes())
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stdin_b.write_all(b"\n")
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stdin_b.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;

                                    let mut response = String::new();
                                    reader_b.read_line(&mut response)
                                        .map_err(|e| NetworkArenaClientError::from(e))?;

                                    stream.write_all(response.as_bytes())
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stream.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                },
                                "white" => {
                                    stdin_w.write_all(command[1].as_bytes())
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stdin_w.write_all(b"\n")
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stdin_w.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;

                                    let mut response = String::new();
                                    reader_w.read_line(&mut response)
                                        .map_err(|e| NetworkArenaClientError::from(e))?;

                                    stream.write_all(response.as_bytes())
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stream.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                },
                                "stats" => {
                                    let win = command[1].parse::<usize>()
                                        .map_err(|_| NetworkArenaClientError::UnexpectedServerResponse)?;
                                    let lose = command[2].parse::<usize>()
                                        .map_err(|_| NetworkArenaClientError::UnexpectedServerResponse)?;
                                    let draw = command[3].parse::<usize>()
                                        .map_err(|_| NetworkArenaClientError::UnexpectedServerResponse)?;
                                    self.stats.0 += win;
                                    self.stats.1 += lose;
                                    self.stats.2 += draw;
                                    stream.write_all(b"ok\n")
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stream.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                },
                                "pieces" => {
                                    let player = command[1].parse::<usize>()
                                        .map_err(|_| NetworkArenaClientError::UnexpectedServerResponse)?;
                                    let opponent = command[2].parse::<usize>()
                                        .map_err(|_| NetworkArenaClientError::UnexpectedServerResponse)?;
                                    self.pieces.0 += player;
                                    self.pieces.1 += opponent;
                                    stream.write_all(b"ok\n")
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stream.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                },
                                "quit" =>  {
                                    stream.write_all(b"ok\n")
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    stream.flush()
                                        .map_err(|e| NetworkArenaClientError::from(e))?;
                                    break;
                                }
                                _ => return Err(NetworkArenaClientError::UnexpectedServerResponse),
                            }
                        } else {
                            return Err(NetworkArenaClientError::UnexpectedServerResponse);
                        }
                        response.clear();
                    }
                }
                Err(e) => return Err(NetworkArenaClientError::from(e)),
                
            }
        }

        process_b.kill()
            .map_err(|e| NetworkArenaClientError::from(e))?;
        process_w.kill()
            .map_err(|e| NetworkArenaClientError::from(e))?;
        process_b.wait()
            .map_err(|e| NetworkArenaClientError::from(e))?;
        process_w.wait()
            .map_err(|e| NetworkArenaClientError::from(e))?;
        Ok(())
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        self.stats
    }

    pub fn get_pieces(&self) -> (usize, usize) {
        self.pieces
    }
}
