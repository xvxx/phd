use async_std::{
    fs,
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn start(addr: &str, root: &str) -> Result<()> {
    let fut = listen(addr, root);
    task::block_on(fut)
}

async fn listen(addr: &str, root: &str) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("-> Connection from: {}", stream.peer_addr()?);
        let root = root.to_string();
        task::spawn(client_loop(stream, root));
    }
    Ok(())
}

async fn client_loop(mut stream: TcpStream, root: String) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    if let Some(Ok(line)) = lines.next().await {
        println!("-> client sent: {:?}", line);
        respond(&mut stream, &line, &root).await?;
    }
    Ok(())
}

async fn respond(stream: &mut TcpStream, selector: &str, root: &str) -> Result<()> {
    let mut response = format!("iYou sent: {}\r\n", selector);

    let mut dir = fs::read_dir(root).await?;

    while let Some(Ok(entry)) = dir.next().await {
        response.push_str(&format!(
            "1{}\t/{}\tlocalhost\t7070\r\n",
            entry.file_name().into_string().unwrap(),
            entry.file_name().into_string().unwrap(),
        ));
    }
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}
