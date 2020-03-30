use chrono::prelude::*;
use clap::{crate_version, App, Arg};
use lognplot::net::TcpClient;
use regex::Regex;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() {
    let matches = App::new("octoprint_serial_logs_lognplot")
                          .version(crate_version!())
                          .about("Read an Octoprint serial log file and send temperature data to a running lognplot instance to plot them.")
                          .arg(Arg::with_name("serial_logs")
                               .help("The serial_logs file to use")
                               .required(true)
                               .index(1))
                          .get_matches();
    // Get filename
    let filename = matches.value_of("serial_logs").unwrap();
    // Open the file
    let file = File::open(filename).expect("file not found");
    // Create the regex
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3}) - Recv: ok T:([0-9]*[.]?[0-9]+) [[:punct:]]([0-9]*[.]?[0-9]+) B:([0-9]*[.]?[0-9]+) [[:punct:]]([0-9]*[.]?[0-9]+)").expect("Could not compile regex");
    // Create the lognplot client
    let mut client = TcpClient::new("127.0.0.1:12345")
        .expect("Could not connect to lognplot server. Is it running?");
    // Create a buffered reader
    let reader = BufReader::new(file);
    // Read the file line by line
    for line in reader.lines().map(|l| l.unwrap()) {
        // Apply regex
        match re.captures(&line) {
            Some(caps) => {
                let stamp = NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M:%S,%f")
                    .expect("Failed to parse date")
                    .timestamp();
                let extruder_temp: f64 = caps[2].parse().unwrap();
                let extruder_setpoint: f64 = caps[3].parse().unwrap();
                let bed_temp: f64 = caps[4].parse().unwrap();
                let bed_setpoint: f64 = caps[5].parse().unwrap();
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
            None => (),
        }
    }
}
