pub mod moderation;

extern crate mailgun_rs;

use anyhow::Result;
use std::collections::HashMap;
use std::env;
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};

pub struct MailOpts {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub template_vars: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentifierAndTokenParams {
    pub identifier: String,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenParam {
    pub token: String,
}

pub async fn send_reset_password(to: String, params: IdentifierAndTokenParams) -> Result<()> {
    let m = Message::new(Email::new(to))
        .set_subject("Password Reset Requested")
        .set_from(Email::new(env::var("PDS_EMAIL_FROM_ADDRESS").unwrap()))
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(format!("Reset Password {}", params.token)),
        );
    let sender = Sender::new(env::var("SENDGRID_API_KEY").unwrap(), None);
    sender.send(&m).await?;
    Ok(())
}

pub async fn send_account_delete(to: String, params: TokenParam) -> Result<()> {
    let m = Message::new(Email::new(to))
        .set_subject("Account Deletion Requested")
        .set_from(Email::new(env::var("PDS_EMAIL_FROM_ADDRESS").unwrap()))
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(format!("Delete Account {}", params.token)),
        );
    let sender = Sender::new(env::var("SENDGRID_API_KEY").unwrap(), None);
    sender.send(&m).await?;
    Ok(())
}

pub async fn send_confirm_email(to: String, params: TokenParam) -> Result<()> {
    let m = Message::new(Email::new(to))
        .set_subject("Email Confirmation")
        .set_from(Email::new(env::var("PDS_EMAIL_FROM_ADDRESS").unwrap()))
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(format!("Confirm Email {}", params.token)),
        );
    let sender = Sender::new(env::var("SENDGRID_API_KEY").unwrap(), None);
    sender.send(&m).await?;
    Ok(())
}

pub async fn send_update_email(to: String, params: TokenParam) -> Result<()> {
    let m = Message::new(Email::new(to))
        .set_subject("Email Update Requested")
        .set_from(Email::new(env::var("PDS_EMAIL_FROM_ADDRESS").unwrap()))
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(format!("Update Email {}", params.token)),
        );
    let sender = Sender::new(env::var("SENDGRID_API_KEY").unwrap(), None);
    sender.send(&m).await?;
    Ok(())
}

pub async fn send_plc_operation(to: String, params: TokenParam) -> Result<()> {
    let m = Message::new(Email::new(to))
        .set_subject("PLC Update Operation Requested")
        .set_from(Email::new(env::var("PDS_EMAIL_FROM_ADDRESS").unwrap()))
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(format!("Plc Operation {}", params.token)),
        );
    let sender = Sender::new(env::var("SENDGRID_API_KEY").unwrap(), None);
    sender.send(&m).await?;
    Ok(())
}
