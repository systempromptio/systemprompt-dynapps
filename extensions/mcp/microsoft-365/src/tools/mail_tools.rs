//! The mail domain's tool table: every `d365_*` mail tool beside
//! `d365_list_mail`, each bound to its function, input, reply and
//! description. Attachment downloads and uploads go through the file layer.

use super::file_tool::file_tools;
use super::graph_tool::graph_tools;
use super::mail_attach::{
    self, AddAttachmentInput, AttachmentIdInput, UploadChunkInput, UploadSessionInput,
};
use super::mail_folders::{
    self, CreateCategoryInput, CreateMailFolderInput, CreateMailRuleInput, FolderIdInput,
};
use super::mail_read::{
    self, ListAttachmentsInput, MessageIdInput, SearchMailInput, SharedMailboxInput, SkipInput,
};
use super::mail_send::{
    self, CreateDraftInput, ForwardMailInput, MoveMailInput, ReplyMailInput, SendMailInput,
    UpdateMailInput,
};
use super::reply::{GraphItem, GraphList};

graph_tools! {
    "d365_get_mail" => mail_read::get_mail, MessageIdInput, GraphItem, mail_read::GET_MAIL_DESCRIPTION;
    "d365_search_mail" => mail_read::search_mail, SearchMailInput, GraphList, mail_read::SEARCH_MAIL_DESCRIPTION;
    "d365_list_mail_folders" => mail_read::list_mail_folders, SkipInput, GraphList, mail_read::LIST_MAIL_FOLDERS_DESCRIPTION;
    "d365_list_mail_attachments" => mail_read::list_mail_attachments, ListAttachmentsInput, GraphList, mail_read::LIST_MAIL_ATTACHMENTS_DESCRIPTION;
    "d365_list_mail_rules" => mail_read::list_mail_rules, SkipInput, GraphList, mail_read::LIST_MAIL_RULES_DESCRIPTION;
    "d365_list_shared_mailbox_messages" => mail_read::list_shared_mailbox_messages, SharedMailboxInput, GraphList, mail_read::LIST_SHARED_MAILBOX_DESCRIPTION;
    "d365_send_mail" => mail_send::send_mail, SendMailInput, GraphItem, mail_send::SEND_MAIL_DESCRIPTION;
    "d365_reply_mail" => mail_send::reply_mail, ReplyMailInput, GraphItem, mail_send::REPLY_MAIL_DESCRIPTION;
    "d365_forward_mail" => mail_send::forward_mail, ForwardMailInput, GraphItem, mail_send::FORWARD_MAIL_DESCRIPTION;
    "d365_create_draft" => mail_send::create_draft, CreateDraftInput, GraphItem, mail_send::CREATE_DRAFT_DESCRIPTION;
    "d365_send_draft" => mail_send::send_draft, MessageIdInput, GraphItem, mail_send::SEND_DRAFT_DESCRIPTION;
    "d365_update_mail" => mail_send::update_mail, UpdateMailInput, GraphItem, mail_send::UPDATE_MAIL_DESCRIPTION;
    "d365_delete_mail" => mail_send::delete_mail, MessageIdInput, GraphItem, mail_send::DELETE_MAIL_DESCRIPTION;
    "d365_move_mail" => mail_send::move_mail, MoveMailInput, GraphItem, mail_send::MOVE_MAIL_DESCRIPTION;
    "d365_create_mail_attachment_upload_session" => mail_attach::create_upload_session, UploadSessionInput, GraphItem, mail_attach::CREATE_UPLOAD_SESSION_DESCRIPTION;
    "d365_delete_mail_attachment" => mail_attach::delete_mail_attachment, AttachmentIdInput, GraphItem, mail_attach::DELETE_MAIL_ATTACHMENT_DESCRIPTION;
    "d365_create_mail_folder" => mail_folders::create_mail_folder, CreateMailFolderInput, GraphItem, mail_folders::CREATE_MAIL_FOLDER_DESCRIPTION;
    "d365_delete_mail_folder" => mail_folders::delete_mail_folder, FolderIdInput, GraphItem, mail_folders::DELETE_MAIL_FOLDER_DESCRIPTION;
    "d365_create_mail_rule" => mail_folders::create_mail_rule, CreateMailRuleInput, GraphItem, mail_folders::CREATE_MAIL_RULE_DESCRIPTION;
    "d365_create_outlook_category" => mail_folders::create_outlook_category, CreateCategoryInput, GraphItem, mail_folders::CREATE_OUTLOOK_CATEGORY_DESCRIPTION;
}

file_tools! {
    "d365_get_mail_attachment" => mail_attach::get_mail_attachment, AttachmentIdInput, GraphItem, mail_attach::GET_MAIL_ATTACHMENT_DESCRIPTION;
    "d365_add_mail_attachment" => mail_attach::add_mail_attachment, AddAttachmentInput, GraphItem, mail_attach::ADD_MAIL_ATTACHMENT_DESCRIPTION;
    "d365_upload_attachment_chunk" => mail_attach::upload_attachment_chunk, UploadChunkInput, GraphItem, mail_attach::UPLOAD_CHUNK_DESCRIPTION;
}
