/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/


#![feature(core)]
#![feature(str_words)]

extern crate core;
extern crate hyper;
extern crate url;
extern crate ansi_term;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::net::SocketAddr::{V4,V6};
use hyper::status::StatusCode;
use hyper::status::StatusClass::{Success,Redirection,ClientError,ServerError};
use hyper::version::HttpVersion::{Http09,Http10,Http11,Http20};
use ansi_term::Colour::{Green,Yellow,Red,Cyan};
use ansi_term::Style::Plain;

fn color_status(status: StatusCode) -> String {
    let s = format!("{}", status);

    match status.class() {
        Success => Green.paint(&s),
        Redirection => Yellow.paint(&s),
        ClientError | ServerError => Red.paint(&s),
        _ => Plain.paint(&s),
    }.to_string()
}

fn color_version(version: hyper::version::HttpVersion) -> String {
    let v = format!("{}", version);

    match version {
        Http09 | Http10 => Yellow.paint(&v),
        Http11 => Green.paint(&v),
        Http20 => Cyan.paint(&v),
    }.to_string()
}

fn color_header(name: String, value: String) -> String {
    let h = format!("{}: {}", name, value);

    match name.as_slice() {
        "Location" => Yellow.paint(&h),
        "Server" | "Via" | "X-Powered-By" | "CF-RAY" => Cyan.paint(&h),
        _ => Plain.paint(&h),
    }.to_string()
}

fn lookup_ips(domain: String) -> std::io::Result<String> {

    let hosts: std::net::LookupHost =  try!(std::net::lookup_host(&domain));

    let mut ips: Vec<String> = Vec::new();
    for host in hosts {
        let ip = match host.unwrap() {
            V4(sa4) => format!("{}", sa4.ip()),
            V6(sa6) => format!("{}", sa6.ip()),
        };
        ips.push(ip);
    }

    ips.sort();
    ips.dedup();

    // For now just return first match
    // Need to prioritize IPv6 over IPv4 plus be able to select one or the
    // other. Fallback from IPv6 to IPv4 with a warning would be nice too.
    Ok(ips[0].clone())
}

fn domain_in_hosts(domain: &String) -> bool {
    let file = match File::open("/etc/hosts") {
        Ok(file) => file,
        Err(..) => return false,
    };

    let buffer = BufReader::new(&file);

    for line in buffer.lines() {

        if line.is_err() {
            continue;
        }

        let entry: &str = &line.unwrap();

        let mut elements: Vec<&str> = entry.words().collect();

        // skip commented lines
        if elements.len() > 0 && (*elements.remove(0)).starts_with("#") {
            continue;
        }

        for e in elements {
            if e == *domain {
                return true;
            }
        }
    }

    false
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: {} <URL>", args[0]);
        return;
    }

    let url_in = if !args[1].starts_with("http://") && !args[1].starts_with("https://") {
        format!("http://{}", args[1])
    } else {
        args[1].clone()
    };

    let mut url = match url::Url::parse(&url_in) {
        Ok(x) => x,
        Err(y) => {
            println!("Invalid input: {}", y);
            return;
        }
    };

    let domain = url.domain().unwrap().to_string();

    let hosts_hack = if domain_in_hosts(&domain) {
        Red.paint(" [/etc/hosts]").to_string()
    } else {
        "".to_string()
    };

    let ip_address = match lookup_ips(domain.to_string()) {
        Ok(a) => a,
        Err(b) => {
            println!("Error: {}", b);
            return;
        }
    };

    {
        let mut dm = url.domain_mut().unwrap();
        dm.clear();
        dm.push_str(&ip_address);
    }

    let mut client = hyper::Client::new();
    client.set_redirect_policy(hyper::client::RedirectPolicy::FollowNone);

    let res = client.get(url)
        .header(hyper::header::UserAgent("hit/0.0.1".to_string()))
        .header(hyper::header::Host {hostname: domain, port: None})
        .send();

    match res {
        Ok(y) => {
            println!("{} {} @ {}{}", color_version(y.version), color_status(y.status), Cyan.paint(&ip_address).to_string(), hosts_hack);

            let mut headers: Vec<(String, String)> = Vec::new();

            for h in y.headers.iter() {
                headers.push((h.name().to_string(), h.value_string()));
            }

            headers.sort();

            for (name, value) in headers {
                println!("\u{25CF} {}", color_header(name, value));
            }
        },
        Err(x) => println!("{}", x),
    }
}
