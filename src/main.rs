extern crate chrono;
extern crate clap;
extern crate lognplot;
extern crate regex;

use chrono::prelude::*;
use clap::{App, Arg};
use lognplot::net::TcpClient;
use regex::Regex;
use std::fs::File;
use std::io::Read;

fn main() {
    let matches = App::new("octoprint_serial_logs_lognplot")
                          .version("1.0")
                          .about("Read an Octoprint serial log file and send temperature data to a running lognplot instance to plot them.")
                          .arg(Arg::with_name("serial_logs")
                               .help("The serial_logs file to use")
                               .required(true)
                               .index(1))
                          .get_matches();
    let filename = matches.value_of("serial_logs").unwrap();
    let mut f = File::open(filename).expect("file not found");
    let mut content = String::new();
    // Warning, load all file in memory...
    f.read_to_string(&mut content)
        .expect("something went wrong reading the file");
    let re = match Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3}) - Recv: ok T:([0-9]*[.]?[0-9]+) [[:punct:]]([0-9]*[.]?[0-9]+) B:([0-9]*[.]?[0-9]+) [[:punct:]]([0-9]*[.]?[0-9]+)") {
      Ok(r) => r,
      Err(e) => {
        println!("Could not compile regex: {}", e);
        std::process::exit(1);
      }
    };

    let mut client = match TcpClient::new("127.0.0.1:12345") {
        Ok(client) => client,
        Err(_e) => {
            println!("Could not connect to lognplot server. Is it running?");
            std::process::exit(1);
        }
    };

    for cap in re.captures_iter(&content) {
        let stamp = NaiveDateTime::parse_from_str(&cap[1], "%Y-%m-%d %H:%M:%S,%f")
            .expect("Failed to parse date")
            .timestamp();
        let extruder_temp: f64 = cap[2].parse().unwrap();
        let extruder_setpoint: f64 = cap[3].parse().unwrap();
        let bed_temp: f64 = cap[4].parse().unwrap();
        let bed_setpoint: f64 = cap[5].parse().unwrap();
        client
            .send_sample("extruder temp", stamp as f64, extruder_temp)
            .unwrap();
        client
            .send_sample("extruder setpoint", stamp as f64, extruder_setpoint)
            .unwrap();
        client
            .send_sample("bed temp", stamp as f64, bed_temp)
            .unwrap();
        client
            .send_sample("bed setpoint", stamp as f64, bed_setpoint)
            .unwrap();
    }
}
