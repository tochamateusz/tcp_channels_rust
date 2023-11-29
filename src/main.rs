use std::fmt::Display;
use std::io::Write;
use std::net::TcpListener;
use std::result;

type Result<T> = result::Result<T, ()>;

struct Sensitive<T>(T);

const SAFE_MODE: bool = false;

impl<T: Display> Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if SAFE_MODE {
            let _var_name = writeln!(f, "[REDACTED]");
        } else {
            let _var_name = writeln!(f, "{inner}", inner = self.0);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let address = "127.0.0.1:6969";

    let tcp_listener = TcpListener::bind(address)
        .map_err(|err| eprint!("ERROR: could not bind {address}: {}", Sensitive(err)))?;

    println!("INFO: listening to {}", Sensitive(address));

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("{s:#?}");
                let _w =
                    writeln!(s, "Hello").map_err(|e| eprintln!("cannot write stream to user {e}"));
            }
            Err(e) => eprintln!("encounter IO error: {e}"),
        }
    }

    println!("Hello, world!: {tcp_listener:#?}");
    Ok(())
}
