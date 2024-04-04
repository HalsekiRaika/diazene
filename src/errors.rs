#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("")]
    CallBackSend,

    #[error("May have passed different type information than what was expected when downcasting from `Any` to type.")]
    DownCastFromAny,
}
