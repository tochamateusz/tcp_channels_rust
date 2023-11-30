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

fn client(stream: Arc<TcpStream>, messages: Sender<Message>) -> Result<()> {
    messages
        .send(Message::ClientConnected(stream.clone()))
        .map_err(|err| eprintln!("ERROR: couldn't send message to server thread: {err}"))?;

    let mut buffer = vec![0; 10];

    loop {
        let n = stream.deref().read(&mut buffer).map_err(|_| {
            let _ = messages.send(Message::ClientDisconnected(stream.clone()));
        })?;

        if n == 0 {
            let _ = messages.send(Message::ClientDisconnected(stream.clone()));
            break Ok(());
        }

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
