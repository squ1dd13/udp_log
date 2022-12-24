use serde::{Deserialize, Serialize};
use std::{env, net, process, str};

#[derive(Clone, Copy, Serialize, Deserialize)]
enum MessageType {
    Normal,
    Error,
    Warning,
    Debug,
}

#[derive(Serialize, Deserialize)]
struct Message {
    module: String,
    msg_type: MessageType,
    string: String,
    time: String,
}

impl Message {
    fn print(&self) {
        let level_str = match self.msg_type {
            MessageType::Normal => "\x1b[32;1m[info]",
            MessageType::Error => "\x1b[31;1m[error]",
            MessageType::Warning => "\x1b[33;1m[warning]",
            MessageType::Debug => "\x1b[34;1m[debug]",
        };

        // This is a direct copy of the format used by VSCode (adapted for Rust).
        //      [date time] [module] [level] Text
        println!(
            "[{}] [{}] {}\x1b[0m {}\x1b[0m",
            self.time, self.module, level_str, self.string
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: udp-log port");
        process::exit(1);
    }

    let address = String::from("0.0.0.0:") + &args[1];
    let socket = net::UdpSocket::bind(&address);

    if let Err(error) = socket {
        eprintln!("Unable to bind socket to '{}'. Error: {}", address, error);
        process::exit(1);
    }

    let socket = socket.unwrap();
    let mut len_buf = [0, 0, 0, 0];

    loop {
        if let Err(error) = socket.peek_from(&mut len_buf) {
            eprintln!("Error reading body length: {}", error);

            // Ignore this message.
            continue;
        }

        let msg_length = u32::from_le_bytes(len_buf);

        let mut payload = vec![0; msg_length as usize];

        if let Err(error) = socket.recv_from(&mut payload) {
            eprintln!("Error reading full payload: {}", error);
            continue;
        }

        let message_bytes = &payload[4..];
        let message = bincode::deserialize::<Message>(message_bytes);

        if let Err(error) = message {
            eprintln!("Error decoding payload: {}", error);
            continue;
        }

        message.unwrap().print();
    }
}
