use crate::interceptor::InterceptorArgs;
use syn::{Ident, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParsedControllerKind {
    Web,
    SocketIo,
    Grpc,
}

pub struct CommonControllerInput {
    pub struct_name: Ident,
    pub base_path: String,
    pub kind: ParsedControllerKind,
    pub fields: Vec<(Ident, Type)>,
    pub interceptors: Vec<InterceptorArgs>,
}
