//! Account messages and command history. Every call makes one explicit request.
use crate::{reads::Operation, CommandOutcome, Error, SaicClient};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageGroup {
    Alarm,
    Command,
    News,
}
impl MessageGroup {
    fn wire(self) -> &'static str {
        match self {
            Self::Alarm => "ALARM",
            Self::Command => "COMMAND",
            Self::News => "NEWS",
        }
    }
}
impl std::str::FromStr for MessageGroup {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "alarm" => Ok(Self::Alarm),
            "command" => Ok(Self::Command),
            "news" => Ok(Self::News),
            _ => Err(Error::Configuration("expected alarm, command or news")),
        }
    }
}

/// Preserve the server's ID type, including strings with leading zeros.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageId {
    Text(String),
    Number(serde_json::Number),
}
impl std::fmt::Debug for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MessageId { .. }")
    }
}
impl MessageId {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Text(s) if !s.trim().is_empty() => Ok(()),
            Self::Number(n) if n.is_i64() || n.is_u64() => Ok(()),
            _ => Err(Error::Configuration(
                "expected a nonempty string or integer message ID",
            )),
        }
    }
}

/// Bulk deletion always requires a group; single-entry actions always require an ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageAction {
    Read(MessageId),
    Delete(MessageId),
    DeleteAll(MessageGroup),
}
impl MessageAction {
    pub(crate) fn operation(&self) -> Result<Operation, Error> {
        let body = match self {
            Self::Read(id) | Self::Delete(id) => {
                id.validate()?;
                serde_json::json!({"actionType": if matches!(self, Self::Read(_)) { "READ" } else { "DELETE" }, "messageId":id})
            }
            Self::DeleteAll(group) => {
                serde_json::json!({"actionType":format!("DELETE_{}", group.wire())})
            }
        };
        Ok(operation(
            "/message/status".into(),
            Method::PUT,
            body.to_string(),
        ))
    }
}
fn operation(path: String, method: Method, body: String) -> Operation {
    Operation {
        path,
        method,
        body: zeroize::Zeroizing::new(body),
        events: false,
        allow_empty_result: true,
    }
}
pub(crate) fn list_operation(
    group: MessageGroup,
    page_num: u32,
    page_size: u32,
) -> Result<Operation, Error> {
    if page_num == 0 || page_size == 0 {
        return Err(Error::Configuration(
            "page number and size must be positive",
        ));
    }
    Ok(operation(
        format!(
            "/message/list?pageNum={page_num}&pageSize={page_size}&messageGroup={}",
            group.wire()
        ),
        Method::GET,
        String::new(),
    ))
}
pub(crate) fn unread_operation() -> Operation {
    operation("/message/unreadCount".into(), Method::GET, String::new())
}
impl SaicClient {
    /// Read exactly one page (numbered from 1). Never marks or deletes messages.
    pub async fn get_message_list(
        &mut self,
        group: MessageGroup,
        page_num: u32,
        page_size: u32,
    ) -> Result<Option<crate::models::MessageResp>, Error> {
        self.message_read(list_operation(group, page_num, page_size)?)
            .await
    }
    pub async fn get_alarm_list(
        &mut self,
        page_num: u32,
        page_size: u32,
    ) -> Result<Option<crate::models::MessageResp>, Error> {
        self.get_message_list(MessageGroup::Alarm, page_num, page_size)
            .await
    }
    pub async fn get_command_list(
        &mut self,
        page_num: u32,
        page_size: u32,
    ) -> Result<Option<crate::models::MessageResp>, Error> {
        self.get_message_list(MessageGroup::Command, page_num, page_size)
            .await
    }
    pub async fn get_news_list(
        &mut self,
        page_num: u32,
        page_size: u32,
    ) -> Result<Option<crate::models::MessageResp>, Error> {
        self.get_message_list(MessageGroup::News, page_num, page_size)
            .await
    }
    pub async fn get_unread_messages_count(
        &mut self,
    ) -> Result<Option<crate::models::MessageResp>, Error> {
        self.message_read(unread_operation()).await
    }
    async fn message_read(
        &mut self,
        operation: Operation,
    ) -> Result<Option<crate::models::MessageResp>, Error> {
        let mut outcome = self
            .execute_operation(&operation, Default::default(), std::future::pending())
            .await?;
        // The current message endpoints explicitly permit absent results.
        outcome.data.get_or_insert(serde_json::Value::Null);
        Self::read_result(outcome)
    }
    /// Apply one typed action. No retries, implicit reads, or status refreshes.
    pub async fn update_message_status(
        &mut self,
        action: MessageAction,
    ) -> Result<CommandOutcome, Error> {
        self.execute_operation(
            &action.operation()?,
            Default::default(),
            std::future::pending(),
        )
        .await
    }
}
