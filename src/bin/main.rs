use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::fs;
use std::time::Duration;
use std::thread;
use webserver::ThreadPool;
use std::sync::{Arc, Mutex};
use std::collections::HashMap; 

struct Cache {
    files: HashMap<String, String>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            files: HashMap::new(),
        }
    }

    fn get(&self, filename: &str) -> Option<&String> {
        self.files.get(filename)
    }

    fn insert(&mut self, filename: String, contents: String) {
        self.files.insert(filename, contents);
    }
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        let mut new_files = HashMap::new();
        for (key, value) in &self.files {
            new_files.insert(key.clone(), value.clone());
        }
        Cache { files: new_files }
    }
}


fn main() {
    let cache = Arc::new(Mutex::new(Cache::new()));

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(4) {
        let stream = stream.unwrap();
        let cache_clone = Arc::clone(&cache);

        pool.execute(move|| {
            handle_connection(stream, cache_clone);
        });

    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream,cache: Arc<Mutex<Cache>>){
    let mut buffer = [0;1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line,filename) =
        if buffer.starts_with(get) {
            ("HTTP/1.1 200 OK", "index.html")
        } else if buffer.starts_with(sleep) {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        };

    let mut cache = cache.lock().unwrap();

    let contents = if let Some(cached_contents) = cache.get(filename) {
        println!("Cache Hit");
        cached_contents.clone() 
    } else {
        let file_contents = fs::read_to_string(&filename).unwrap();
        cache.insert(filename.to_string(), file_contents.clone()); 
        
        file_contents
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    
}