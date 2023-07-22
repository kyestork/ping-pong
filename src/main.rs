use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[derive(Debug)]
enum AppError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO: {}", e),
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Listener for a shutdown signal
    let (shutdown_send, mut shutdown_recv) = mpsc::channel(1);
    let shutdown_task = tokio::spawn(async move {
        let _ = shutdown_recv.recv().await;
        println!("Shutting down...");
    });

    // Listener for TCP connections
    let tcp_task = tokio::spawn(tcp_server_listen(shutdown_send.clone()));

    // Concurrent task selection
    tokio::select! {
        _ = shutdown_task => {},
        _ = tcp_task => {}
    }

    drop(shutdown_send);

    println!("All connections closed. Server has shutdown.");
    Ok(())
}


async fn tcp_server_listen(shutdown_send: mpsc::Sender<()>) -> Result<(), AppError> {
    // Listen on port 8080
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        //Accept a new connection
        let (socket, _) = listener.accept().await?;
        println!("Accepting a connection.");
        
        //Spawn new task
        tokio::spawn(tcp_server_recv(socket, shutdown_send.clone()));

    }
}

async fn tcp_server_recv(socket: TcpStream, shutdown_send: mpsc::Sender<()>) -> Result<(), AppError> {
    println!("Receiving data.");
    let mut socket = BufReader::new(socket);

    loop {

        let mut buffer = "".to_owned();
        let bytes_read = match socket.read_line(&mut buffer).await {
            Ok(0) => {
                // Disconnect
                break;
            }
            Ok(n) => n,
            Err(e) => { return Err(AppError::IoError(e)) }
        };

        println!("Received {} bytes: {:?}", bytes_read, &buffer);

        match buffer.trim() {
            "PING" => {
                println!("-- PING --");
                if let Err(e) = socket.write_all(b"PONG\r\n").await { return Err(AppError::IoError(e)); }
            },
            "STOP" => {
                println!("Received remote shutdown command.");
                let _ = socket.write_all(b"SURE\r\n").await.unwrap();
                let _ = shutdown_send.send(()).await;
                // Disconnect
                break;
            },
            _ => {}
        }


    }

    println!("Client disconnected.");
    Ok(())
}