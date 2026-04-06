mod interceptor;

pub use interceptor::*;
pub use sword_macros::GrpcError;

use std::any::TypeId;
use sword_core::State;

pub use tonic;
pub use tonic::{Code, Request, Response, Status, Streaming};

use crate::application::engines::grpc::GrpcServiceRegistry;
use crate::prelude::ControllerSpec;

pub type GrpcResult<T> = Result<Response<T>, Status>;

#[derive(Clone)]
pub struct GrpcBodyLimitValue {
    pub max_decoding_message_size: usize,
    pub max_encoding_message_size: usize,
}

impl Default for GrpcBodyLimitValue {
    fn default() -> Self {
        Self {
            max_decoding_message_size: 4 * 1024 * 1024,
            max_encoding_message_size: 4 * 1024 * 1024,
        }
    }
}

/// Trait implemented by gRPC controllers declared with `#[controller(kind = Controller::Grpc, ...)]`.
pub trait GrpcController: ControllerSpec {
    fn service_name() -> &'static str;
}

#[derive(Clone)]
pub struct GrpcControllerRegistrar {
    pub controller_id: TypeId,
    pub service_name: &'static str,
    pub build: fn(&State),
    pub register: fn(&State, &mut GrpcServiceRegistry),
}

inventory::collect!(GrpcControllerRegistrar);

#[derive(Clone, Copy)]
pub struct GrpcReflectionDescriptorRegistrar {
    pub encoded_file_descriptor_set: &'static [u8],
}

inventory::collect!(GrpcReflectionDescriptorRegistrar);
