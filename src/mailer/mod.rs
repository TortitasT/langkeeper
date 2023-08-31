use dotenvy::dotenv;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use maud::html;

use crate::{jwt::generate_auth_jwt, logger::log, models::User};

pub async fn send_mail(to: &str, subject: &str, content: &String, html: bool) {
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
        .subject(subject);

    let message = if html {
        message.html_body(content)
    } else {
        message.text_body(content)
    };

    match SmtpClientBuilder::new(address.clone(), port)
        .credentials((username, password))
        .implicit_tls(true)
        .connect()
        .await
    {
        Ok(mut client) => match client.send(message).await {
            Ok(_) => log(
                format!("Successfully sent email to {}", to).as_str(),
                crate::logger::LogLevel::Info,
            ),
            Err(e) => log(
                format!("Failed to send email to {}: {}", to, e).as_str(),
                crate::logger::LogLevel::Error,
            ),
        },
        Err(e) => log(
            format!("Failed to connect to {}: {}", address, e).as_str(),
            crate::logger::LogLevel::Error,
        ),
    };
}

pub fn send_verification_email(user: &User) {
    dotenv().ok();

    let app_url = std::env::var("APP_URL").unwrap();

    let user_email = user.email.clone();
    let user_name = user.name.clone();

    let token = generate_auth_jwt(user);
    let url = format!("{}/users/verify/{}", app_url, token.unwrap());
    let email_html = html!(
    p {
        "Hi " (user_name) "!"
    }
    p {
        "Please verify your email by clicking on the following link: "
        a href=(url) {
            "Verify!"
        }
    }
    );

    log(
        format!("Sending verification email to {}", user_email).as_str(),
        crate::logger::LogLevel::Info,
    );

    actix_rt::spawn(async move {
        send_mail(
            &user_email,
            "Verify your account at Langkeeper",
            &email_html.into_string(),
            true,
        )
        .await;
    });
}
