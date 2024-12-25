use crate::arena::core::{Arena, Player};
use crate::arena::error::{ClientManagerError, NetworkArenaClientError, NetworkArenaServerError};
use crate::board::core::Turn;
use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

const SUPER_COMMAND_MARKER: &str = "##SUPER##";
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const BUF_SIZE: usize = 1024;

struct StreamBuffer {
    stream: Arc<Mutex<TcpStream>>,
    black_lines: VecDeque<Vec<u8>>,
    white_lines: VecDeque<Vec<u8>>,
}

impl StreamBuffer {
    fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
        StreamBuffer {
            stream,
            black_lines: VecDeque::new(),
            white_lines: VecDeque::new(),
        }
    }

    fn process_line(&mut self, line: &str) {
        let black_marker = "black ";
        let white_marker = "white ";

        if line.starts_with(black_marker) {
            self.black_lines
                .push_back(line.strip_prefix(black_marker).unwrap().as_bytes().to_vec());
        } else if line.starts_with(white_marker) {
            self.white_lines
                .push_back(line.strip_prefix(white_marker).unwrap().as_bytes().to_vec());
        }
    }

    fn read_next_line(&mut self) -> io::Result<Option<()>> {
        let mut stream = self.stream.lock().unwrap();
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }

        let data = String::from_utf8_lossy(&buf[..n]).to_string();

        drop(stream);

        for line in data.lines() {
            let line = format!("{}\n", line);
            self.process_line(&line);
        }

        Ok(Some(()))
    }

    fn read_black(&mut self) -> io::Result<Option<Vec<u8>>> {
        if let Some(line) = self.black_lines.pop_front() {
            return Ok(Some(line));
        }

        while let Ok(Some(())) = self.read_next_line() {
            if let Some(line) = self.black_lines.pop_front() {
                return Ok(Some(line));
            }
        }

        Ok(None)
    }

    fn read_white(&mut self) -> io::Result<Option<Vec<u8>>> {
        if let Some(line) = self.white_lines.pop_front() {
            return Ok(Some(line));
        }

        while let Ok(Some(())) = self.read_next_line() {
            if let Some(line) = self.white_lines.pop_front() {
                return Ok(Some(line));
            }
        }

        Ok(None)
    }
}

struct StreamWriter {
    stream: Arc<Mutex<TcpStream>>,
    is_black: bool,
}

impl Write for StreamWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let data = String::from_utf8_lossy(buf);
        let command = if self.is_black {
            format!("{} black {}", SUPER_COMMAND_MARKER, data)
        } else {
            format!("{} white {}", SUPER_COMMAND_MARKER, data)
        };
        let mut stream = self.stream.lock().unwrap();
        stream.write_all(command.as_bytes())?;
        stream.flush()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct StreamReader {
    buffer: Arc<Mutex<StreamBuffer>>,
    current_data: Option<Vec<u8>>,
    current_pos: usize,
    is_black: bool,
}

impl StreamReader {
    fn new(buffer: Arc<Mutex<StreamBuffer>>, is_black: bool) -> Self {
        StreamReader {
            buffer,
            current_data: None,
            current_pos: 0,
            is_black,
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(data) = self.current_data.as_ref() {
            if self.current_pos < data.len() {
                let n = std::cmp::min(buf.len(), data.len() - self.current_pos);
                buf[..n].copy_from_slice(&data[self.current_pos..self.current_pos + n]);
                self.current_pos += n;
                return Ok(n);
            }
        }

        self.current_data = None;
        self.current_pos = 0;
        let mut stream_buffer = self.buffer.lock().unwrap();
        let result = if self.is_black {
            stream_buffer.read_black()?
        } else {
            stream_buffer.read_white()?
        };

        match result {
            Some(data) => {
                let n = std::cmp::min(buf.len(), data.len());
                buf[..n].copy_from_slice(&data[..n]);
                if n < data.len() {
                    self.current_data = Some(data);
                    self.current_pos = n;
                }
                Ok(n)
            }
            None => Ok(0),
        }
    }
}

impl BufRead for StreamReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        while self.current_data.is_none() {
            let mut stream_buffer = self.buffer.lock().unwrap();
            self.current_data = if self.is_black {
                stream_buffer.read_black()?
            } else {
                stream_buffer.read_white()?
            };
            self.current_pos = 0;

            if self.current_data.is_none() {
                let eof = stream_buffer.read_next_line()?.is_none();
                if eof {
                    break;
                }
            }
        }

        Ok(match &self.current_data {
            Some(data) => &data[self.current_pos..],
            None => &[],
        })
    }

    fn consume(&mut self, amt: usize) {
        if let Some(data) = &self.current_data {
            self.current_pos = std::cmp::min(self.current_pos + amt, data.len());
            if self.current_pos >= data.len() {
                self.current_data = None;
                self.current_pos = 0;
            }
        }
    }
}

#[derive(Debug)]
struct ClientManager {
    clients: [Option<TcpStream>; 2],
}

type PlayerPair = (
    Player<StreamWriter, StreamReader>,
    Player<StreamWriter, StreamReader>,
);

impl ClientManager {
    fn new() -> Self {
        ClientManager {
            clients: [None, None],
        }
    }

    fn add_client(&mut self, stream: TcpStream) -> Result<(), ClientManagerError> {
        for i in 0..2 {
            if self.clients[i].is_none() {
                stream.set_read_timeout(Some(READ_TIMEOUT))?;
                // .map_err(ClientManagerError::from)?;
                self.clients[i] = Some(stream);
                println!("Client {} connected", i);
                return Ok(());
            }
        }
        Err(ClientManagerError::NoMoreClients)
    }

    fn is_full(&self) -> bool {
        self.clients.iter().all(|x| x.is_some())
    }

    fn is_ready(&mut self) -> Result<bool, ClientManagerError> {
        for stream in self.clients.iter_mut() {
            match stream.as_mut() {
                Some(stream) => {
                    stream.write_all(SUPER_COMMAND_MARKER.as_bytes())?;
                    stream.write_all(b" isready\n")?;
                    stream.flush()?;
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

    fn get_players(&self) -> Result<Vec<PlayerPair>, ClientManagerError> {
        let stream1 = self.clients[0]
            .as_ref()
            .ok_or(ClientManagerError::ClientNotExists)?;
        let stream2 = self.clients[1]
            .as_ref()
            .ok_or(ClientManagerError::ClientNotExists)?;
        let stream1 = Arc::new(Mutex::new(stream1.try_clone()?));
        let stream2 = Arc::new(Mutex::new(stream2.try_clone()?));
        let stream_buffer1 = Arc::new(Mutex::new(StreamBuffer::new(stream1.clone())));
        let stream_buffer2 = Arc::new(Mutex::new(StreamBuffer::new(stream2.clone())));
        let player1b = Player::new(
            StreamWriter {
                stream: stream1.clone(),
                is_black: true,
            },
            StreamReader::new(stream_buffer1.clone(), true),
        );
        let player2w = Player::new(
            StreamWriter {
                stream: stream2.clone(),
                is_black: false,
            },
            StreamReader::new(stream_buffer2.clone(), false),
        );
        let player2b = Player::new(
            StreamWriter {
                stream: stream2.clone(),
                is_black: true,
            },
            StreamReader::new(stream_buffer2.clone(), true),
        );
        let player1w = Player::new(
            StreamWriter {
                stream: stream1.clone(),
                is_black: false,
            },
            StreamReader::new(stream_buffer1.clone(), false),
        );
        Ok(vec![(player1b, player2w), (player2b, player1w)])
    }

    fn send_results(
        &mut self,
        results: (usize, usize, usize),
        pieces: (usize, usize),
    ) -> Result<(), ClientManagerError> {
        for (i, stream) in self.clients.as_mut().iter_mut().enumerate() {
            match stream.as_mut() {
                Some(stream) => {
                    stream.write_all(SUPER_COMMAND_MARKER.as_bytes())?;
                    if i == 0 {
                        stream.write_all(
                            format!(" stats {} {} {}\n", results.0, results.1, results.2)
                                .as_bytes(),
                        )?;
                    } else {
                        stream.write_all(
                            format!(" stats {} {} {}\n", results.1, results.0, results.2)
                                .as_bytes(),
                        )?;
                    }
                    stream.flush()?;
                    let mut buffer = [0; BUF_SIZE];
                    let mut response = String::new();
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        }
                        Err(e) => return Err(ClientManagerError::from(e)),
                    }
                    if response.trim() != "ok" {
                        return Err(ClientManagerError::UnexpectedResponse);
                    }
                    stream.write_all(SUPER_COMMAND_MARKER.as_bytes())?;
                    if i == 0 {
                        stream
                            .write_all(format!(" pieces {} {}\n", pieces.0, pieces.1).as_bytes())?;
                    } else {
                        stream
                            .write_all(format!(" pieces {} {}\n", pieces.1, pieces.0).as_bytes())?;
                    }
                    stream.flush()?;
                    let mut buffer = [0; BUF_SIZE];
                    let mut response = String::new();
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        }
                        Err(e) => return Err(ClientManagerError::from(e)),
                    }
                    if response.trim() != "ok" {
                        return Err(ClientManagerError::UnexpectedResponse);
                    }
                }
                None => return Err(ClientManagerError::ClientNotExists),
            }
        }
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), ClientManagerError> {
        for stream in self.clients.iter_mut() {
            match stream {
                Some(stream) => {
                    stream.write_all(SUPER_COMMAND_MARKER.as_bytes())?;
                    stream.write_all(b" quit\n")?;
                    stream.flush()?;
                    let mut buffer = [0; BUF_SIZE];
                    let mut response = String::new();
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        }
                        Err(e) => return Err(ClientManagerError::from(e)),
                    }
                    if response.trim() != "ok" {
                        return Err(ClientManagerError::UnexpectedResponse);
                    }
                }
                None => return Err(ClientManagerError::ClientNotExists),
            }
        }
        println!("Clients disconnected");
        Ok(())
    }

    fn clear(&mut self) {
        self.clients = [None, None];
    }
}

#[derive(Debug)]
pub struct NetworkArenaServer {
    game_per_iter: usize,
    client_manager: ClientManager,
}

impl NetworkArenaServer {
    pub fn new(game_per_iter: usize) -> Result<Self, NetworkArenaServerError> {
        if game_per_iter % 2 != 0 {
            return Err(NetworkArenaServerError::GameNumberInvalid);
        }
        Ok(NetworkArenaServer {
            game_per_iter,
            client_manager: ClientManager::new(),
        })
    }

    pub fn start(&mut self, addr: String, port: u16) -> Result<(), NetworkArenaServerError> {
        let listener = TcpListener::bind(format!("{}:{}", addr, port))?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => match self.client_manager.add_client(stream) {
                    Ok(_) => {
                        if self.client_manager.is_full() {
                            self.play()?;
                            self.client_manager.disconnect()?;
                            self.client_manager.clear();
                        }
                    }
                    Err(e) => {
                        return Err(NetworkArenaServerError::from(e));
                    }
                },
                Err(e) => {
                    return Err(NetworkArenaServerError::from(e));
                }
            }
        }
        Ok(())
    }

    fn play(&mut self) -> Result<(), NetworkArenaServerError> {
        if !self.client_manager.is_ready()? {
            return Err(NetworkArenaServerError::ClientNotReady);
        }
        let players = self.client_manager.get_players()?;
        let mut arena = Arena::new(players);
        arena.play_n(self.game_per_iter)?;

        let (p1_win, p2_win, draw) = arena.get_stats();
        let (p1_pieces, p2_pieces) = arena.get_pieces();

        self.client_manager
            .send_results((p1_win, p2_win, draw), (p1_pieces, p2_pieces))?;
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

    fn start_process(
        command: &[String],
        turn: Turn,
    ) -> Result<(Child, ChildStdin, BufReader<ChildStdout>), std::io::Error> {
        let mut cmd = Command::new(&command[0]);
        for arg in command.iter().skip(1) {
            cmd.arg(arg);
        }

        match turn {
            Turn::Black => cmd.arg("BLACK"),
            Turn::White => cmd.arg("WHITE"),
        };

        let mut process = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

        let mut stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        // ping-pong test
        writeln!(stdin, "ping")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Write error"))?;
        stdin
            .flush()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Flush error"))?;

        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Read error"))?;

        if response.trim() != "pong" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ping-pong test failed",
            ));
        }

        Ok((process, stdin, reader))
    }

    pub fn connect(&mut self, addr: String, port: u16) -> Result<(), NetworkArenaClientError> {
        let mut stream = TcpStream::connect(format!("{}:{}", addr, port))?;
        stream.set_read_timeout(Some(READ_TIMEOUT))?;

        let (mut process_b, mut stdin_b, mut reader_b) =
            NetworkArenaClient::start_process(&self.command, Turn::Black)?;
        let (mut process_w, mut stdin_w, mut reader_w) =
            NetworkArenaClient::start_process(&self.command, Turn::White)?;

        let mut buffer = [0; BUF_SIZE];
        let mut response = String::new();

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => return Err(NetworkArenaClientError::ConnectionBroken),
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    if response.ends_with("\n") {
                        for line in response.clone().lines() {
                            if line.starts_with(SUPER_COMMAND_MARKER) {
                                let command_line =
                                    line.trim_start_matches(SUPER_COMMAND_MARKER).trim();
                                let command: Vec<&str> = command_line.split_whitespace().collect();
                                match command[0] {
                                    "isready" => {
                                        if command.len() != 1 {
                                            return Err(
                                                NetworkArenaClientError::UnexpectedServerResponse,
                                            );
                                        }
                                        stream.write_all(b"readyok\n")?;
                                        stream.flush()?;
                                    }
                                    "black" => {
                                        if command.len() != 2 {
                                            return Err(
                                                NetworkArenaClientError::UnexpectedServerResponse,
                                            );
                                        }
                                        stdin_b.write_all(command[1].as_bytes())?;
                                        stdin_b.write_all(b"\n")?;
                                        stdin_b.flush()?;

                                        let mut response = String::new();
                                        reader_b.read_line(&mut response)?;

                                        let response_with_color =
                                            format!("{} {}", "black", response);
                                        stream.write_all(response_with_color.as_bytes())?;
                                        stream.flush()?;
                                    }
                                    "white" => {
                                        if command.len() != 2 {
                                            return Err(
                                                NetworkArenaClientError::UnexpectedServerResponse,
                                            );
                                        }
                                        stdin_w.write_all(command[1].as_bytes())?;
                                        stdin_w.write_all(b"\n")?;
                                        stdin_w.flush()?;

                                        let mut response = String::new();
                                        reader_w.read_line(&mut response)?;

                                        let response_with_color =
                                            format!("{} {}", "white", response);
                                        stream.write_all(response_with_color.as_bytes())?;
                                        stream.flush()?;
                                    }
                                    "stats" => {
                                        if command.len() != 4 {
                                            return Err(
                                                NetworkArenaClientError::UnexpectedServerResponse,
                                            );
                                        }
                                        let win = command[1].parse::<usize>().map_err(|_| {
                                            NetworkArenaClientError::UnexpectedServerResponse
                                        })?;
                                        let lose = command[2].parse::<usize>().map_err(|_| {
                                            NetworkArenaClientError::UnexpectedServerResponse
                                        })?;
                                        let draw = command[3].parse::<usize>().map_err(|_| {
                                            NetworkArenaClientError::UnexpectedServerResponse
                                        })?;
                                        self.stats.0 += win;
                                        self.stats.1 += lose;
                                        self.stats.2 += draw;
                                        stream.write_all(b"ok\n")?;
                                        stream.flush()?;
                                    }
                                    "pieces" => {
                                        if command.len() != 3 {
                                            return Err(
                                                NetworkArenaClientError::UnexpectedServerResponse,
                                            );
                                        }
                                        let player = command[1].parse::<usize>().map_err(|_| {
                                            NetworkArenaClientError::UnexpectedServerResponse
                                        })?;
                                        let opponent =
                                            command[2].parse::<usize>().map_err(|_| {
                                                NetworkArenaClientError::UnexpectedServerResponse
                                            })?;
                                        self.pieces.0 += player;
                                        self.pieces.1 += opponent;
                                        stream.write_all(b"ok\n")?;
                                        stream.flush()?;
                                    }
                                    "quit" => {
                                        if command.len() != 1 {
                                            return Err(
                                                NetworkArenaClientError::UnexpectedServerResponse,
                                            );
                                        }
                                        stream.write_all(b"ok\n")?;
                                        stream.flush()?;
                                        process_b.kill()?;
                                        process_w.kill()?;
                                        process_b.wait()?;
                                        process_w.wait()?;
                                        return Ok(());
                                    }
                                    _ => {
                                        return Err(
                                            NetworkArenaClientError::UnexpectedServerResponse,
                                        )
                                    }
                                }
                            } else {
                                return Err(NetworkArenaClientError::UnexpectedServerResponse);
                            }
                        }
                        response.clear();
                    }
                }
                Err(e) => return Err(NetworkArenaClientError::from(e)),
            }
        }
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        self.stats
    }

    pub fn get_pieces(&self) -> (usize, usize) {
        self.pieces
    }
}
