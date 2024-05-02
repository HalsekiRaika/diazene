use anyid::AnyId;

#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("Actor with this identifier: {id} has already been activated.")]
    AlreadySpawned {
        id: AnyId
    },

    #[error("")]
    CallBackSend,

    #[error("May have passed different type information than what was expected when downcasting from `Any` to type.")]
    DownCastFromAny,
}
