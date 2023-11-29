use std::fmt::Display;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver};
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
    New,
}
fn server(_message: Receiver<Message>) -> Result<()> {
    Ok(())
}

fn client(mut stream: TcpStream) -> Result<()> {
    let _w = writeln!(stream, "Hello").map_err(|e| eprintln!("cannot write stream to user {e}"));
    Ok(())
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
                thread::spawn(|| client(s));
            }
            Err(e) => eprintln!("encounter IO error: {e}"),
        }
    }

    println!("Hello, world!: {tcp_listener:#?}");
    Ok(())
}
