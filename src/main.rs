use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::{result, thread};

type Result<T> = result::Result<T, ()>;

struct Sensitive<T>(T);

const SAFE_MODE: bool = false;

impl<T: Display> Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        if SAFE_MODE {
            let _var_name = writeln!(f, "[REDACTED]");
        } else {
            let _var_name = inner.fmt(f);
        }
        Ok(())
    }
}

enum Message {
    ClientConnected(Arc<TcpStream>),
    ClientDisconnected(Arc<TcpStream>),
    New {
        bytes: Vec<u8>,
        conn: Arc<TcpStream>,
    },
}

#[derive(Debug)]
struct Client {
    conn: Arc<TcpStream>,
}

fn server(messages: Receiver<Message>) -> Result<()> {
    let mut clients = HashMap::new();
    loop {
        let msg = messages.recv().expect("The server reciver is hung up");
        match msg {
            Message::ClientConnected(author) => {
                let addr = author.peer_addr().expect("TODO: cache the peer");
                let client = Client {
                    conn: author.clone(),
                };

                let message = format!(
                    "Hello client: {}",
                    client.conn.peer_addr().expect("Lack of addr")
                );

                client
                    .conn
                    .as_ref()
                    .write(message.as_bytes())
                    .map_err(|err| eprintln!("ERROR: couldn't write bytest {err}"))?;

                let _inserted = clients.insert(addr, client);
            }
            Message::ClientDisconnected(author) => {
                let addr = author.peer_addr().expect("TODO: cache the peer");
                clients.remove(&addr);
            }
            Message::New { bytes, conn } => {
                let con_addr = conn.peer_addr().expect("TODO: cache the peer");
                for (addr, client) in clients.iter() {
                    if *addr != con_addr {
                        client
                            .conn
                            .as_ref()
                            .write(&bytes)
                            .map_err(|err| eprintln!("ERROR: couldn't write bytes {err}"))?;
                    };
                }
            }
        }
    }
}

fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<()> {
    const fn val(c: u8) -> Result<u8> {
        match c {
            b'A'..=b'F' => Ok(c - b'A' + 10),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'0'..=b'9' => Ok(c - b'0'),
            _ => todo!(),
        }
    }
    let data = data.as_ref();

    if data.len() % 2 != 0 {
        eprintln!("data is not divided by 2");
        return Ok(());
    }

    println!("{} != {}", data.len() / 2, out.len());
    if data.len() / 2 != out.len() {
        eprintln!("to small buffer");
        return Ok(());
    }

    for (i, byte) in out.iter_mut().enumerate() {
        *byte = val(data[2 * i])? << 4 | val(data[2 * i + 1])?;
    }

    Ok(())
}

fn client(stream: Arc<TcpStream>, messages: Sender<Message>) -> Result<()> {
    messages
        .send(Message::ClientConnected(stream.clone()))
        .map_err(|err| eprintln!("ERROR: couldn't send message to server thread: {err}"))?;

    let mut buffer = vec![0; 10];

    loop {
        let n = stream.deref().read(&mut buffer).map_err(|_| {
            let _ = messages.send(Message::ClientDisconnected(stream.clone()));
        })?;

        const DATA: &str = "48656c6c6f20636c69656e743a203132372e302e302e313a3535313336";
        let mut byte_array = [0; DATA.len() / 2];

        let _ = decode_to_slice(DATA, &mut byte_array);

        println!("TEST: {}", String::from_utf8(byte_array.to_vec()).unwrap());

        if n == 0 {
            let _ = messages.send(Message::ClientDisconnected(stream.clone()));
            break Ok(());
        }

        let client_addr = stream.deref().peer_addr().map_err(|e| {
            eprintln!("cannot get address:{e}");
        })?;

        let message_content =
            String::from_utf8(buffer.clone()).expect("buffer cannot be converter to utf8");

        println!(
            "Client: {}, filling the buffer:{}",
            client_addr, message_content
        );

        let _ = messages
            .send(Message::New {
                bytes: buffer[0..n].to_vec(),
                conn: stream.clone(),
            })
            .map_err(|err| eprintln!("ERROR: couldn't send message to server thread: {err}"));

        buffer.fill(0);
    }
}

fn main() -> Result<()> {
    let address = "127.0.0.1:6969";

    let tcp_listener = TcpListener::bind(address)
        .map_err(|err| eprint!("ERROR: could not bind {address}: {}", Sensitive(err)))?;

    println!("INFO: listening to {}", Sensitive(address));

    let (message_sender, message_revciver) = channel::<Message>();
    thread::spawn(|| server(message_revciver));

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(s) => {
                println!("{s:#?}");
                let message_sender = message_sender.clone();
                thread::spawn(|| client(Arc::new(s), message_sender));
            }
            Err(e) => eprintln!("encounter IO error: {e}"),
        }
    }

    println!("Hello, world!: {tcp_listener:#?}");
    Ok(())
}
