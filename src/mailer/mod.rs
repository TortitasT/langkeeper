use dotenvy::dotenv;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};

pub async fn send_text_mail(to: &str, subject: &str, text: &str) {
    dotenv().ok();

    let address = std::env::var("MAILER_ADDRESS").unwrap();
    let port = std::env::var("MAILER_PORT")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let email = std::env::var("MAILER_EMAIL").unwrap();
    let username = std::env::var("MAILER_USERNAME").unwrap();
    let password = std::env::var("MAILER_PASSWORD").unwrap();

    let message = MessageBuilder::new()
        .from(("Langkeeper", email.as_str()))
        .to(to)
        .subject(subject)
        .text_body(text);

    match SmtpClientBuilder::new(address.clone(), port)
        .credentials((username, password))
        .implicit_tls(true)
        .connect()
        .await
    {
        Ok(mut client) => match client.send(message).await {
            Ok(_) => println!("Email sent successfully to {}!", to),
            Err(e) => println!("Error sending email to {}: {}", to, e),
        },
        Err(e) => println!("Error connecting to {}: {}", address, e),
    };
}
