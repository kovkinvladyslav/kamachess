use crate::models::{Message, SendMessageRequest, TelegramResponse, Update};
use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct TelegramApi {
    client: reqwest::Client,
    base_url: String,
}

impl TelegramApi {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: format!("https://api.telegram.org/bot{}", token),
        }
    }

    pub async fn send_message(&self, chat_id: i64, reply_to: i64, text: &str) -> Result<i64> {
        let url = format!("{}/sendMessage", self.base_url);
        let body = SendMessageRequest {
            chat_id,
            text: text.to_string(),
            reply_to_message_id: Some(reply_to),
            parse_mode: Some("HTML".to_string()),
        };

        let resp: TelegramResponse<Message> = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            let error_msg = resp
                .description
                .unwrap_or_else(|| "sendMessage failed".to_string());
            return Err(anyhow!("Telegram API error: {}", error_msg));
        }

        Ok(resp
            .result
            .ok_or_else(|| anyhow!("Telegram API error: missing result in response"))?
            .message_id)
    }

    pub async fn send_photo(
        &self,
        chat_id: i64,
        reply_to: Option<i64>,
        caption: &str,
        png: Vec<u8>,
    ) -> Result<i64> {
        let url = format!("{}/sendPhoto", self.base_url);
        let mut form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .text("caption", caption.to_string())
            .text("parse_mode", "HTML".to_string())
            .part(
                "photo",
                reqwest::multipart::Part::bytes(png)
                    .file_name("board.png")
                    .mime_str("image/png")?,
            );

        if let Some(reply_to) = reply_to {
            form = form.text("reply_to_message_id", reply_to.to_string());
        }

        let resp: TelegramResponse<Message> = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            let error_msg = resp
                .description
                .unwrap_or_else(|| "sendPhoto failed".to_string());
            return Err(anyhow!("Telegram API error: {}", error_msg));
        }

        Ok(resp
            .result
            .ok_or_else(|| anyhow!("Telegram API error: missing result in response"))?
            .message_id)
    }

    pub async fn delete_message(&self, chat_id: i64, message_id: i64) -> Result<()> {
        let url = format!("{}/deleteMessage", self.base_url);
        let body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
        });

        let resp: TelegramResponse<serde_json::Value> = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            let error_msg = resp
                .description
                .unwrap_or_else(|| "deleteMessage failed".to_string());
            // Don't fail if message is already deleted or too old
            if error_msg.contains("message to delete not found")
                || error_msg.contains("message can't be deleted")
            {
                return Ok(());
            }
            return Err(anyhow!("Telegram API error: {}", error_msg));
        }

        Ok(())
    }

    pub async fn get_updates(&self, offset: Option<i64>, timeout: i32) -> Result<Vec<Update>> {
        let url = format!("{}/getUpdates", self.base_url);
        let mut params = vec![("timeout", timeout.to_string())];
        if let Some(offset) = offset {
            params.push(("offset", offset.to_string()));
        }

        let resp: TelegramResponse<Vec<Update>> = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            let error_msg = resp
                .description
                .unwrap_or_else(|| "getUpdates failed".to_string());
            return Err(anyhow!("Telegram API error: {}", error_msg));
        }

        Ok(resp.result.unwrap_or_default())
    }
}
