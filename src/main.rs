use clap::{AppSettings, Clap};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    usize,
};
use tracing::{event, span, Level};
use tracing_subscriber;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp, about="proxy an address. eg.: proxy --listen=0.0.0.0:3307 --to=localhost:3306")]
struct Opts {
    #[clap(short, long, default_value = "0.0.0.0:20807")]
    listen: String,
    #[clap(short, long)]
    target: String,
    #[clap(short, long, default_value = "4096")]
    buffer_size: usize,
}

// read from `from`  and write to `to`
fn proxy(mut from: TcpStream, mut to: TcpStream, buf_size: usize) {
    event!(Level::INFO, msg = "proxy:start");
    loop {
        let mut buf: Vec<u8> = vec![0; buf_size];
        match from.read(buf.as_mut_slice()) {
            Ok(n) => {
                if n == 0 {
                    event!(Level::INFO, msg = "read:end");
                    break;
                }
                match to.write(&buf[..n]) {
                    Ok(n) => {
                        event!(Level::TRACE, msg = "tran", bytes = n);
                    }
                    Err(err) => {
                        let err_str = err.to_string();
                        event!(Level::ERROR, msg = "tran:error", err = err_str.as_str());
                        break;
                    }
                }
            }
            Err(err) => {
                let err_str = err.to_string();
                event!(Level::ERROR, msg = "read:error", err = err_str.as_str());
                break;
            }
        }
    }

    event!(Level::INFO, msg = "proxy:end");
}

fn handle_stream(
    input_stream: TcpStream,
    target: &str,
    id: usize,
    buffer_size: usize,
) -> Result<(), std::io::Error> {
    let ipt_rx_stream = input_stream;
    let ipt_tx_stream = ipt_rx_stream.try_clone()?;

    let opt_rx_stream = TcpStream::connect(target)?;
    let opt_tx_stream = opt_rx_stream.try_clone()?;

    // ipt -> opt
    std::thread::Builder::new()
        .name(format!("proxy[{}] ==>", id))
        .spawn(move || {
            let span = span!(Level::ERROR, "==>", id);
            let _guard = span.enter();
            proxy(ipt_rx_stream, opt_tx_stream, buffer_size);
        })?;

    // opt->ipt
    std::thread::Builder::new()
        .name(format!("proxy[{}] <==", id))
        .spawn(move || {
            let span = span!(Level::ERROR, "<==", id);
            let _guard = span.enter();
            proxy(opt_rx_stream, ipt_tx_stream, buffer_size);
        })?;

    Ok(())
}

fn main() {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();

    event!(
        Level::INFO,
        listen = opts.listen.as_str(),
        target = opts.target.as_str(),
        buffer_size = opts.buffer_size
    );

    let listener = TcpListener::bind(opts.listen).expect("listen failed");

    let mut next_id = 1;
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        match handle_stream(stream, &opts.target, next_id, opts.buffer_size) {
            Err(err) => {
                let err_str = err.to_string();
                event!(Level::ERROR, id = next_id, err = err_str.as_str());
            }
            Ok(()) => {}
        };
        next_id += 1;
    }
}
