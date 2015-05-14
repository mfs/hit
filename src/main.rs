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

extern crate hyper;
extern crate url;
extern crate ansi_term;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::net::Ipv4Addr;
use std::str::FromStr;
use hyper::status::StatusCode;
use hyper::status::StatusClass::{Success,Redirection,ClientError,ServerError};
use hyper::version::HttpVersion::{Http09,Http10,Http11,Http20};
use ansi_term::Colour::{Green,Yellow,Red,Cyan};
use ansi_term::Style::Plain;
use std::process::Command;

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

    let n: &str = &name;

    match n {
        "Location" => Yellow.paint(&h),
        "Server" | "Via" | "X-Powered-By" | "CF-RAY" => Cyan.paint(&h),
        _ => Plain.paint(&h),
    }.to_string()
}

fn is_ipv4addr(addr: &str) -> bool {
    Ipv4Addr::from_str(addr).is_ok()
}

fn lookup_ip(domain: String) -> Result<(String, bool), String> {
    match domain_in_hosts(&domain) {
        Some(s) => {
            return Ok((s, true));
        },
        None => {},
    }

    // Until we get a stable lookup_host or DNS library just shell out to dig.
    // Yeah, this isn't great.
    let output = Command::new("dig").arg("+short").arg(&domain).output().unwrap();

    let result = String::from_utf8(output.stdout).unwrap();

    if result == "" {
       return Err(format!("could not resolve host: {}", &domain).to_string());
    }

    let ips: Vec<&str> = result.split(char::is_whitespace).filter(|&x| x != "" && is_ipv4addr(x)).collect();

    Ok((ips[0].to_string(), false))
}

fn domain_in_hosts(domain: &String) -> Option<String> {
    let file = match File::open("/etc/hosts") {
        Ok(file) => file,
        Err(..) => return None,
    };

    let buffer = BufReader::new(&file);

    for line in buffer.lines() {

        if line.is_err() {
            continue;
        }

        let entry: &str = &line.unwrap();

        let mut elements: Vec<&str> = entry.split(char::is_whitespace).filter(|&x| x != "").collect();

        // skip lines with less than 2 elements
        if elements.len() < 2 { continue; }

        let ip = elements.remove(0);
        if ip.starts_with("#") { continue; }

        for e in elements {
            if e == *domain {
                return Some(ip.to_string());
            }
        }
    }

    None
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

    let (ip_address, lt) = match lookup_ip(domain.to_string()) {
        Ok(a) => a,
        Err(b) => {
            println!("Error: {}", b);
            return;
        }
    };

    let hosts_hack = if lt {
        Red.paint(" [/etc/hosts]").to_string()
    } else {
        "".to_string()
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
