use dynwork_chats::error::ChatsError;
use dynwork_chats::ids::IdError;
use dynwork_chats::store::StoreError;

#[test]
fn every_error_has_a_stable_key() {
    assert_eq!(ChatsError::Id(IdError::Cursor).key(), "invalid_cursor");
    assert_eq!(ChatsError::Id(IdError::Thread).key(), "invalid_id");
    assert_eq!(ChatsError::Id(IdError::Message).key(), "invalid_id");
    assert_eq!(
        ChatsError::Store(StoreError::Corrupt("dynwork_chat_threads.id")).key(),
        "internal"
    );
}
