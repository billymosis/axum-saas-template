use std::sync::Arc;

use log::debug;
use reqwest::Client;
use serde_json::json;

use crate::{
    config::Config,
    http::Result,
    http::{models::email::EmailResponse, Error},
};

pub async fn send_verification_email(
    username: &str,
    email: &str,
    client: Client,
    token: &str,
    config: Arc<Config>,
) -> Result<()> {
    let link = format!("{}/api/auth/verify-email/{}", config.host, token);
    let body = &json!(
    {
        "template_key": &config.email_verification_template_key,
        "from":
        {
            "address": &config.email_sender_address,
            "name": "noreply"
        },
        "to":
        [
            {
            "email_address":
                {
                    "address": email,
                    "name": username,
                }
            }
        ],
        "merge_info": {"verifyLink": link,"email": &email},
    }
    );
    let request = client
        .post(&config.email_service_url_template)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", &config.email_key)
        .json(body);

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let email_response: EmailResponse = response.json().await?;
                debug!("{:?}", email_response);
                Ok(())
            } else {
                Err(Error::Anyhow(anyhow::anyhow!("Error on email service")))
            }
        }
        Err(error) => Err(Error::Anyhow(anyhow::anyhow!(
            "Error on email service {}",
            error
        ))),
    }
}

pub async fn send_reset_password_email(
    username: &str,
    email: &str,
    client: Client,
    token: &str,
    config: Arc<Config>,
) -> Result<()> {
    let link = format!("{}/api/auth/reset-password/{}", config.host, token);
    let body = &json!(
     {
         "template_key": &config.email_reset_password_template_key,
         "from":
         {
             "address": &config.email_sender_address,
             "name": "noreply"
         },
         "to":
         [
             {
             "email_address":
                 {
                     "address": email,
                     "name": username,
                 }
             }
         ],
    "merge_info": {
    "password_reset_link": link,
    "name": email,
    "team": &config.company,
    "product_name": &config.company,
    "username": username,
    },

     }

         );
    let request = client
        .post(&config.email_service_url_template)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", &config.email_key)
        .json(body);

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                let email_response: EmailResponse = response.json().await?;
                debug!("{:?}", email_response);
                Ok(())
            } else {
                Err(Error::Anyhow(anyhow::anyhow!("Error on email service")))
            }
        }
        Err(error) => {
            println!("ERROR HEERE {:?}", error);
            Err(Error::Anyhow(anyhow::anyhow!(
                "Error on email service {}",
                error
            )))
        }
    }
}
