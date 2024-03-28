#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("")]
    CallBackSend,
    
    #[error("")]
    DownCastFromAny,
}