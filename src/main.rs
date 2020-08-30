use structopt::StructOpt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::stream::StreamExt;
use std::thread::sleep;
use tokio::time::Duration;
use std::convert::TryFrom;
use regex::Regex;

const PING_FREQUENCY: u64 =  100;

#[derive(Debug)]
pub enum Command {
    Line {
        content: String,
    },
    Ping,
    Eof,
}

pub async fn reader(mut tx: Sender<Command>) {
    let mut lines = BufReader::new(stdin()).lines().map(|l| l.unwrap());
    loop {
        match lines.next().await {
            Some(line) => tx.send(Command::Line{content: line}).await.unwrap(),
            None => {
                tx.send(Command::Eof).await.unwrap();
                return
            },
        }
    }
}

pub async fn sleeper(mut tx: Sender<Command>) {
    loop {
        sleep(Duration::from_millis(PING_FREQUENCY));
        if tx.send(Command::Ping).await.is_err() {
            return
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_slf4j() {
        assert_eq!(1, 1);
    }
}


#[derive(Debug, StructOpt)]
#[structopt(name = "multigrep", about = "A multiline-aware grep, enabling grepping over slf4j/log4j logs and more")]
struct Opt {
    #[structopt(short, long, default_value = "\\d{4}-\\d{2}-\\d{2}")]
    start: String,

    #[structopt(short, long, default_value = "1000")]
    timeout: i32,

    pattern: String,

    file: Option<String>,
}

fn flush(buf: &mut Vec<String>, pat: &Regex) {
    if buf.len() == 0 {
        return;
    }
    if pat.is_match(buf.get(0).unwrap()) {
        println!("{}", buf.join("\n"));
    }
    buf.clear();
}

#[tokio::main]
pub async fn main() {
    let args = Opt::from_args();
    let start_re = Regex::new(&format!("^{}", &args.start)).expect("Failed to parse start regex");
    let pat_re = Regex::new(&args.pattern).expect("Failed to parse pattern regex");
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    tokio::spawn(async move {
        reader(tx).await;
    });
    tokio::spawn(async move {
        sleeper(tx2).await;
    });
    let mut buf: Vec<String> = vec![];
    let mut pings = 0;
    while let Some(message) = rx.recv().await {
        match message {
            Command::Line{content} => {
                if start_re.is_match(&content) {
                    flush(&mut buf, &pat_re);
                    pings = 0;
                }
                buf.push(content);
            }
            Command::Ping => {
                pings += 1;
                if pings > args.timeout/(i32::try_from(PING_FREQUENCY).unwrap()) {
                    flush(&mut buf, &pat_re);
                    pings = 0;
                }
            },
            Command::Eof => {
                flush(&mut buf, &pat_re);
                return
            },
        }
    }
}

