use std::{
    fs::read_to_string,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use parallel_webserver::ThreadPool;

fn main() {
    let tcp = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(5);
    for stream in tcp.incoming() {
        pool.execute(|| {
            handle_connection(stream.unwrap());
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let bufreader = BufReader::new(&stream);
    let a = bufreader.lines().next().unwrap().unwrap();
    let (requst_format, filename) = match a.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(3));
            ("HTTP/1.1 200 OK", "sleep.html")
        }
        _ => ("HTTP/1.1 404 NOT_FOUND ", "notfound.html"),
    };
    let content = read_to_string(filename).unwrap();
    let len = content.len();
    let resp_format = format!("{requst_format}\r\nContent-Length:{len}\r\n\r\n{content}");
    stream.write_all(resp_format.as_bytes()).unwrap();
    println!("{requst_format},{filename}");
}
