use crate::AppResult;

use std::{
    net::SocketAddr,
    sync::Arc
};
use tokio::{
    net::{TcpListener, TcpStream},
    io::{BufReader, AsyncBufReadExt, AsyncWriteExt}
};
use rspotify::{
    prelude::*,
    scopes,
    AuthCodeSpotify,
    Credentials,
    OAuth, Config, clients::mutex::Mutex
};

pub async fn oauth_client() -> AppResult<impl OAuthClient> {
    let oauth = OAuth {
        redirect_uri: String::from("http://localhost:8888/callback"),
        scopes: scopes!(
            "user-library-read",
            "playlist-read-private",
            "playlist-read-collaborative",
            "user-read-playback-state",
            "user-read-currently-playing",
            "user-modify-playback-state",
            "user-read-playback-position",
            "user-top-read",
            "user-read-recently-played"
        ),
        ..Default::default()
    };

    let creds = Credentials::from_env()
        .ok_or("Couldn't load environment variables RSPOTIFY_CLIENT_ID and RSPOTIFY_CLIENT_SECRET")?;

    let mut spotify = AuthCodeSpotify::with_config(
        creds,
        oauth,
        Config {
            token_cached: true,
            token_refreshing: true,
            ..Default::default()
        }
    );

    match spotify.read_token_cache(false).await.ok() {
        Some(token) => {
            spotify.token = Arc::new(Mutex::new(token));
        },

        None => {
            let url = spotify.get_authorize_url(false)?;
            let code = get_code_from_user(&spotify, url.as_str()).await?;

            spotify.request_token(code.as_str()).await?;
            spotify.write_token_cache().await?;
        }
    };

    Ok(spotify)
}

async fn get_code_from_user(client: &impl OAuthClient, url: &str) -> AppResult<String> {
    match webbrowser::open(url) {
        Ok(_) => println!("Opened {} in your browser.", url),
        Err(why) => eprintln!(
            "Error when trying to open an URL in your browser: {:?}. \
            Please navigate here manually: {}",
            why, url
        ),
    }

    let addr = "127.0.0.1:8888".parse::<SocketAddr>()?;
    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            let (mut stream, _) = listener.accept().await?;
            let (reader, _) = stream.split();

            let mut buf = String::new();
            let mut buf_reader = BufReader::new(reader);
            buf_reader.read_line(&mut buf).await?;

            let header = buf
                .split_whitespace()
                .collect::<Vec<&str>>();

            println!("{}", header[1]);

            let code = client
                .parse_response_code(format!("{}{}", "http://localhost:8888", header[1]).as_str())
                .ok_or("Unable to parse the response code")?;

            respond_with_success(&mut stream).await?;

            Ok(code)
        },

        Err(_) => {
            println!("Please enter the URL you were redirected to: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            let code = client
                .parse_response_code(&input)
                .ok_or("Unable to parse the response code")?;

            Ok(code)
        }
    }
}

async fn respond_with_success(stream: &mut TcpStream) -> AppResult<()> {
    let contents = String::from("<script>window.close();</script>");
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);

    stream.write(response.as_bytes()).await?;
    stream.flush().await?;

    return Ok(())
}
