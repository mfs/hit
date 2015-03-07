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

extern crate core;
extern crate hyper;
extern crate ansi_term;

use std::env;
use hyper::status::StatusCode;
use hyper::status::StatusClass::{Success,Redirection,ClientError,ServerError};
use hyper::version::HttpVersion::{Http09,Http10,Http11,Http20};
use hyper::header::HeaderView;
use ansi_term::Colour::{Green,Yellow,Red,Cyan};

fn color_status(status: StatusCode) -> String {
    let s = format!("{}", status);

    match status.class() {
        Success => Green.paint(&s).to_string(),
        Redirection => Yellow.paint(&s).to_string(),
        ClientError | ServerError => Red.paint(&s).to_string(),
        _ => s,
    }
}

fn color_version(version: hyper::version::HttpVersion) -> String {
    let v = format!("{}", version);

    match version {
        Http09 | Http10 => Yellow.paint(&v),
        Http11 => Green.paint(&v),
        Http20 => Cyan.paint(&v),
    }.to_string()
}

fn color_header(hv: HeaderView) -> String {
    let h = format!("{}: {}", hv.name(), hv.value_string());

    match hv.name() {
        "Location" => Yellow.paint(&h).to_string(),
        "Server" | "Via" | "X-Powered-By" | "CF-RAY" => Cyan.paint(&h).to_string(),
        _ => h,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: {} <URL>", args[0]);
        return;
    }

    let mut client = hyper::Client::new();
    client.set_redirect_policy(hyper::client::RedirectPolicy::FollowNone);

    let url = if !args[1].starts_with("http://") && !args[1].starts_with("https://") {
        format!("http://{}", args[1])
    } else {
        args[1].clone()
    };

    let res = client.get(url.as_slice()).send();

    match res {
        Ok(y) => {
            println!("{} {}", color_version(y.version), color_status(y.status));

            for h in y.headers.iter() {
                println!("\u{25CF} {}", color_header(h));
            }
        },
        Err(x) => println!("{}", x),
    }
}
