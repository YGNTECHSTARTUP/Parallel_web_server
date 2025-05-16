use std::{
    collections::HashMap,
    fs::read_to_string,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use parallel_webserver::ThreadPool;

fn main() {
    let cache: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let count = Arc::new(Mutex::new(0));
    let cache_hit = Arc::new(Mutex::new(0));
    let tcp = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(5);

    for stream in tcp.incoming() {
        let cachee = Arc::clone(&cache);
        let countee = Arc::clone(&count);
        let hits = Arc::clone(&cache_hit);
        pool.execute(|| {
            handle_connection(stream.unwrap(), cachee, countee, hits);
        });

        println!("Request count is  {:?}", count.lock().unwrap());

        println!("Cache Hit  is  {:?}", cache_hit.lock().unwrap());
    }
}

fn handle_connection(
    mut stream: TcpStream,
    cache: Arc<Mutex<HashMap<String, String>>>,
    count: Arc<Mutex<usize>>,
    hits: Arc<Mutex<usize>>,
) {
    let bufreader = BufReader::new(&stream);
    let a = bufreader.lines().next().unwrap().unwrap();
    let (requst_format, filename) = match a.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(3));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT_FOUND ", "notfound.html"),
    };

    let content = {
        let mut cache_cloned = cache.lock().unwrap();
        if let Some(cached) = cache_cloned.get(filename) {
            *hits.lock().unwrap() += 1;
            cached.clone()
        } else {
            cache_cloned.insert(filename.to_string(), read_to_string(filename).unwrap());
            read_to_string(filename).unwrap()
        }
    };

    let len = content.len();
    let resp_format = format!("{requst_format}\r\nContent-Length:{len}\r\n\r\n{content}");
    stream.write_all(resp_format.as_bytes()).unwrap();
    *count.lock().unwrap() += 1;
}
