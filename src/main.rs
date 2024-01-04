use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use qjsonrs::sync::{Stream, TokenIterator};


const EXAMPLE_JSON: &str = "./mars_weather.json";
const BUFFER_SIZE: usize = 1024;

fn main() {
    read_json().unwrap();
}

fn read_json() -> anyhow::Result<()> {
    let file = File::open(EXAMPLE_JSON)?;
    let mut stream = Stream::from_read(file)?;

    loop {
        match stream.next()? {
            None => {
                break;
            }
            Some(x) => {
                println!("Token: {:?}", x);
            }
        }
    }
    Ok(())
}
