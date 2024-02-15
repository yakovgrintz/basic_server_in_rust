use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{spawn, JoinHandle};
use std::{fs, thread};

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(number_of_thread: usize) -> ThreadPool {
        assert!(number_of_thread > 0);
        let (sender, receiver) = mpsc::channel();
        let mut workers = Vec::with_capacity(number_of_thread);
        let reciver = Arc::new(Mutex::new(receiver));
        for id in 0..number_of_thread {
            workers.push(Worker::new(id, Arc::clone(&reciver)));
        }
        ThreadPool { workers, sender }
    }
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = spawn(move || loop {
            let job = receiver
                .lock()
                .expect("Another treadmill is already locked")
                .recv()
                .unwrap();
            println!("job :{} get mission", id);
            job();
        });
        Worker { id, thread }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;
fn main() {
    let ip_address = "127.0.0.1:7878";
    let listener = TcpListener::bind(ip_address).unwrap();
    let thread_pool = ThreadPool::new(thread::available_parallelism().unwrap().get() / 2);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread_pool.execute(|| handle_connection(stream));
    }
}
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    // let http_request: Vec<_> = buf_reader
    //     .lines()
    //     .map(|result| result.unwrap())
    //     .take_while(|line| !line.is_empty())
    //     .collect();
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, file_name) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),

        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let contents = fs::read_to_string(format!("./html_files/{file_name}")).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
