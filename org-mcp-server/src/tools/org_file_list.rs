use org_core::OrgModeError;
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorCode},
    schemars, tool, tool_router,
};

use crate::core::OrgModeRouter;

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ListFilesRequest {
    #[schemars(description = "Filter results by tags (optional)")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "Maximum number of files to return (optional)")]
    pub limit: Option<usize>,
    #[schemars(description = "Search all silos including discovered repo docs (default: false)")]
    pub all_silos: Option<bool>,
}

#[tool_router(router = "tool_router_list_files", vis = "pub(crate)")]
impl OrgModeRouter {
    #[tool(
        name = "org-file-list",
        description = "List all the org files defined in the org-mode configuration",
        annotations(title = "org-file-list tool")
    )]
    async fn tool_list_files(
        &self,
        Parameters(ListFilesRequest {
            tags,
            limit,
            all_silos,
        }): Parameters<ListFilesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let org_mode = self.org_mode.lock().await;

        if all_silos.unwrap_or(false) {
            // Search all silos (primary + extra + discovered repos)
            match org_mode.list_files_all_silos(tags.as_deref(), limit) {
                Ok(files) => match Content::json(files) {
                    Ok(serialized) => Ok(CallToolResult::success(vec![serialized])),
                    Err(e) => Err(McpError {
                        code: ErrorCode::INTERNAL_ERROR,
                        message: format!("Failed to serialize files: {e}").into(),
                        data: None,
                    }),
                },
                Err(e) => {
                    let error_code = match &e {
                        OrgModeError::InvalidDirectory(_) => ErrorCode::INVALID_PARAMS,
                        OrgModeError::WalkError(_) => ErrorCode::INTERNAL_ERROR,
                        _ => ErrorCode::INTERNAL_ERROR,
                    };
                    Err(McpError {
                        code: error_code,
                        message: format!("Failed to list files from all silos: {e}").into(),
                        data: None,
                    })
                }
            }
        } else {
            // Search only primary directory (original behavior)
            match org_mode.list_files(tags.as_deref(), limit) {
                Ok(files) => match Content::json(files) {
                    Ok(serialized) => Ok(CallToolResult::success(vec![serialized])),
                    Err(e) => Err(McpError {
                        code: ErrorCode::INTERNAL_ERROR,
                        message: format!("Failed to serialize files: {e}").into(),
                        data: None,
                    }),
                },
                Err(e) => {
                    let error_code = match &e {
                        OrgModeError::InvalidDirectory(_) => ErrorCode::INVALID_PARAMS,
                        OrgModeError::WalkError(_) => ErrorCode::INTERNAL_ERROR,
                        _ => ErrorCode::INTERNAL_ERROR,
                    };
                    Err(McpError {
                        code: error_code,
                        message: format!("Failed to list files: {e}").into(),
                        data: None,
                    })
                }
            }
        }
    }
}
