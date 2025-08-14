use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: IssueFields,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueFields {
    pub summary: String,
    pub status: IssueStatus,
    pub assignee: Option<IssueAssignee>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueStatus {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueAssignee {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraSearchResponse {
    pub issues: Vec<JiraIssue>,
    pub total: u32,
    #[serde(rename = "startAt")]
    pub start_at: u32,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorklogComment {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: u32,
    pub content: Vec<WorklogParagraph>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorklogParagraph {
    #[serde(rename = "type")]
    pub paragraph_type: String,
    pub content: Vec<WorklogText>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorklogText {
    #[serde(rename = "type")]
    pub text_type: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorklogVisibility {
    #[serde(rename = "type")]
    pub visibility_type: String,
    pub identifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorklogRequest {
    pub comment: String,
    pub started: String,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorklogResponse {
    pub id: String,
    #[serde(rename = "issueId")]
    pub issue_id: String,
    pub started: String,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: u32,
}
