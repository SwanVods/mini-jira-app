use reqwest;
use std::collections::HashMap;

use crate::jira_types::*;

#[derive(Clone)]
pub struct JiraClient {
    pub base_url: String,
    pub email: String,
    pub access_token: String,
    client: reqwest::Client,
}

impl JiraClient {
    pub fn new(base_url: String, email: String, access_token: String) -> Self {
        // Create client with SSL verification disabled for corporate environments
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            base_url,
            email,
            access_token,
            client,
        }
    }

    /// Get issues assigned to the current user
    pub async fn get_assigned_issues(&self) -> Result<Vec<JiraIssue>, Box<dyn std::error::Error>> {
        let url = format!("{}/rest/api/3/search", self.base_url);
        
        let mut params = HashMap::new();
        params.insert("jql", "assignee=currentUser()");
        params.insert("fields", "summary,status,assignee");
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .basic_auth(&self.email, Some(&self.access_token))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("JIRA API error: {}", response.status()).into());
        }

        let search_response: JiraSearchResponse = response.json().await?;
        Ok(search_response.issues)
    }

    /// Create a worklog entry for a specific issue
    pub async fn create_worklog(
        &self,
        issue_key: &str,
        description: &str,
        started: &str,
        time_spent_seconds: u32,
        visibility: Option<WorklogVisibility>,
    ) -> Result<WorklogResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/rest/api/3/issue/{}/worklog", self.base_url, issue_key);
        
        let worklog_request = WorklogRequest {
            comment: WorklogComment {
                doc_type: "doc".to_string(),
                version: 1,
                content: vec![WorklogParagraph {
                    paragraph_type: "paragraph".to_string(),
                    content: vec![WorklogText {
                        text_type: "text".to_string(),
                        text: description.to_string(),
                    }],
                }],
            },
            started: started.to_string(),
            time_spent_seconds,
            visibility,
        };

        let response = self.client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .basic_auth(&self.email, Some(&self.access_token))
            .json(&worklog_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("JIRA API error: {}", response.status()).into());
        }

        let worklog_response: WorklogResponse = response.json().await?;
        Ok(worklog_response)
    }

    /// Helper function to convert time string (like "2h", "30m") to seconds
    pub fn parse_time_to_seconds(time_str: &str) -> Result<u32, Box<dyn std::error::Error>> {
        if time_str.is_empty() {
            return Err("Time string is empty".into());
        }

        let time_str = time_str.trim();
        let (number_part, unit_part) = if time_str.ends_with('h') {
            (&time_str[..time_str.len()-1], "h")
        } else if time_str.ends_with('m') {
            (&time_str[..time_str.len()-1], "m")
        } else if time_str.ends_with('d') {
            (&time_str[..time_str.len()-1], "d")
        } else {
            return Err("Invalid time format. Use 'h' for hours, 'm' for minutes, 'd' for days".into());
        };

        let number: f32 = number_part.parse()?;
        
        let seconds = match unit_part {
            "h" => (number * 3600.0) as u32,
            "m" => (number * 60.0) as u32,
            "d" => (number * 8.0 * 3600.0) as u32, // Assuming 8 hours per day
            _ => return Err("Invalid time unit".into()),
        };

        Ok(seconds)
    }

    /// Test connection to JIRA
    pub async fn test_connection(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let url = format!("{}/rest/api/3/myself", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .basic_auth(&self.email, Some(&self.access_token))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}