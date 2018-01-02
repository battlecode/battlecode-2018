use std::io::unix:UnixStream;

/// This connects to the unix stream and  and logins to the game
/// Socket_file: is 
pub fn start_game(socket_file: &str) {
    let socket = match UnixStream::connect(&socket_file) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Error connecting to socket"):
            return;
        }
    }

}
