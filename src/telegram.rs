/*!
* This module interfaces with the Telegram API to send
* access requests to my phone
*/

use std::collections::HashMap;

use reqwest::Client;
use serde_json::Value;

use crate::error::Error;

async fn send_access_request(name: String) -> Result<i64, Error> {
    let json = serde_json::json!({
        "text": format!("{name} has requested access"),
        "chat_id": crate::config("TEL_USR_ID")?,
        "parse_mode": "MarkdownV2",
        "reply_markup": {
        "inline_keyboard": [
            [
                {
                    "text": "allow",
                    "callback_data": "allow",
                },
                {
                    "text": "block",
                    "callback_data": "block",
                }
            ]
        ]
        }
    });

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        crate::config("TEL_BOT_KEY")?
    );

    let client = Client::new();

    let message_id = if let Ok(response) = client.post(url).json(&json).send().await {
        if let Ok(resp_json) = response.json::<HashMap<String, Value>>().await {
            resp_json
                .get("result")
                .and_then(|result| result.get("message_id"))
                .and_then(|id| id.as_i64())
        } else {
            None
        }
    } else {
        None
    };

    println!("Got message_id: {message_id:?}");

    // TODO: update the error type
    message_id.ok_or(Error::WrongSecret)
}

async fn get_answer(_message_id: i64) -> Result<bool, Error> {
    // TODO: send the (possibly multiple) request(s) to get the last answer
    // HMM:  maybe add parameter which is the "request id" to match the response on
    Ok(true)
}

pub async fn request_access(name: String) -> Result<bool, Error> {
    // send telegram request
    let message_id = send_access_request(name).await?;

    // get user inline keyboard result
    get_answer(message_id).await
}
