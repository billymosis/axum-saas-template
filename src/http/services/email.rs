use std::sync::Arc;

use log::debug;
use reqwest::Client;
use serde_json::json;

use crate::{config::Config, http::Error, http::Result};

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
            "address": "noreply@kevoucher.com",
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
            let text = response.json().await?;
            debug!("{:?}", text);
            if status.is_success() {
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
             "address": "noreply@kevoucher.com",
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
    "team": "kevoucher.com",
    "product_name": "kevoucher.com",
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
    println!("{:?}", body);

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await?;
            debug!("{:?}", text);
            if status.is_success() {
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