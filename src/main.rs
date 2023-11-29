use std::fmt::Display;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
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
    ClientConnected,
    ClientDisconnected,
    New(Vec<u8>),
}
fn server(_message: Receiver<Message>) -> Result<()> {
    Ok(())
}

fn client(mut stream: TcpStream, messages: Sender<Message>) -> Result<()> {
    messages
        .send(Message::ClientConnected)
        .map_err(|err| eprintln!("ERROR: couldn't send message to server thread: {err}"))?;

    let mut buffer = vec![0; 64];

    loop {
        let n = stream.read(&mut buffer).map_err(|_| {
            let _ = messages.send(Message::ClientDisconnected);
        })?;

        let _ = messages
            .send(Message::New(buffer[0..n].to_vec()))
            .map_err(|err| eprintln!("ERROR: couldn't send message to server thread: {err}"));
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
                thread::spawn(|| client(s, message_sender));
            }
            Err(e) => eprintln!("encounter IO error: {e}"),
        }
    }

    println!("Hello, world!: {tcp_listener:#?}");
    Ok(())
}
