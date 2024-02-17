use anyhow::Context;
use fast_socks5::client::Config;
use fast_socks5::{client::Socks5Stream, Result};
use log::{debug, error, info};
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use proxied_pop3::pop3::POP3Result::{POP3Message, POP3Uidl};
use proxied_pop3::pop3::POP3Stream;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    request_pop3_mail().await
}

async fn request_pop3_mail() -> Result<()> {
    dotenv::dotenv().ok();
    let mut socks;
    let mut config = Config::default();
    config.set_skip_auth(false);

    info!("Connecting to the SOCKS server...");

    socks = Socks5Stream::connect_with_password(
        std::env::var("SOCKS_SERVER_AND_PORT").unwrap(),
        std::env::var("TARGET_DOMAIN").unwrap(),
        std::env::var("TARGET_SERVER_PORT").unwrap().parse().unwrap(),
        std::env::var("SOCKS_USER").unwrap(),
        std::env::var("SOCKS_PASSWORD").unwrap(),
        config,
    ).await?;

    let stream = socks.get_socket().into_std().unwrap();
    stream.set_nonblocking(false).unwrap();
    stream.set_nodelay(false).unwrap();
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();

    info!("Connected to the SOCKS server");

    let ssl_connector = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())
        .context("Can't create SSL Connector")?
        .build();

    let mut pop3_stream = POP3Stream::connect_using_socket(
        stream,
        Some(ssl_connector),
        &std::env::var("TARGET_DOMAIN").unwrap(),
    )?;

    info!("Connected to the POP3 server");

    let res = pop3_stream.login(
        &std::env::var("POP3_USER").unwrap(),
        &std::env::var("POP3_PASSWORD").unwrap(),
    );

    info!("Login response: {:?}", res);

    // print list of emails
    let res = pop3_stream.uidl(None);

    let mut mails = match res {
        POP3Uidl { emails_metadata } => emails_metadata,
        _ => vec![]
    };

    let message = mails.pop().unwrap();

    let res = pop3_stream.retr(message.message_id);

    info!("Retr response: {:?}", res);

    let content = match res {
        POP3Message { raw } => raw,
        _ => vec![]
    };

    // combine the content vec into a single string
    let content = content.iter().fold(String::new(), |mut acc, line| {
        acc.push_str(&line);
        acc
    });

    info!("Content: {}", content);


    Ok(())
}