//! The `OneDrive` domain's tool table: each `d365_*` drive tool bound to its
//! function, input, reply and description, and its title and behaviour
//! class from the Belgian `_annotations.py`, save `d365_upload_file` (it
//! replaces a file of the same name) and `d365_share_file` (it opens the file
//! to others), destructive here. File content, downloads and uploads go
//! through the file layer.

use super::drive::{
    self, DriveItemInput, ListDrivesInput, ListFilesInput, ListPermissionsInput, SearchFilesInput,
};
use super::drive_content::{self, UploadFileInput};
use super::drive_write::{self, CreateFolderInput, MoveFileInput, ShareFileInput};
use super::file_tool::file_tools;
use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use super::{ToolKind, ToolSpec};

pub(super) const DRIVE_SPECS: &[ToolSpec] = &[
    ToolSpec::new("d365_list_drives", "List drives", ToolKind::Read),
    ToolSpec::new("d365_list_files", "List files", ToolKind::Read),
    ToolSpec::new("d365_search_files", "Search files", ToolKind::Read),
    ToolSpec::new("d365_get_file_content", "Get file content", ToolKind::Read),
    ToolSpec::new(
        "d365_get_file_metadata",
        "Get file metadata",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_file_permissions",
        "List file permissions",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_download_file", "Download file", ToolKind::Read),
    ToolSpec::new("d365_upload_file", "Upload file", ToolKind::Destructive),
    ToolSpec::new("d365_create_folder", "Create folder", ToolKind::Write),
    ToolSpec::new("d365_share_file", "Share file", ToolKind::Destructive),
    ToolSpec::new("d365_move_file", "Move file", ToolKind::Write),
    ToolSpec::new("d365_delete_file", "Delete file", ToolKind::Destructive),
];

graph_tools! {
    "d365_list_drives" => drive::list_drives, ListDrivesInput, GraphList, drive::LIST_DRIVES_DESCRIPTION;
    "d365_list_files" => drive::list_files, ListFilesInput, GraphList, drive::LIST_FILES_DESCRIPTION;
    "d365_search_files" => drive::search_files, SearchFilesInput, GraphList, drive::SEARCH_FILES_DESCRIPTION;
    "d365_get_file_metadata" => drive::get_file_metadata, DriveItemInput, GraphItem, drive::GET_FILE_METADATA_DESCRIPTION;
    "d365_list_file_permissions" => drive::list_file_permissions, ListPermissionsInput, GraphList, drive::LIST_FILE_PERMISSIONS_DESCRIPTION;
    "d365_create_folder" => drive_write::create_folder, CreateFolderInput, GraphItem, drive_write::CREATE_FOLDER_DESCRIPTION;
    "d365_share_file" => drive_write::share_file, ShareFileInput, GraphItem, drive_write::SHARE_FILE_DESCRIPTION;
    "d365_move_file" => drive_write::move_file, MoveFileInput, GraphItem, drive_write::MOVE_FILE_DESCRIPTION;
    "d365_delete_file" => drive_write::delete_file, DriveItemInput, GraphItem, drive_write::DELETE_FILE_DESCRIPTION;
}

file_tools! {
    "d365_get_file_content" => drive_content::get_file_content, DriveItemInput, GraphItem, drive_content::GET_FILE_CONTENT_DESCRIPTION;
    "d365_download_file" => drive_content::download_file, DriveItemInput, GraphItem, drive_content::DOWNLOAD_FILE_DESCRIPTION;
    "d365_upload_file" => drive_content::upload_file, UploadFileInput, GraphItem, drive_content::UPLOAD_FILE_DESCRIPTION;
}
