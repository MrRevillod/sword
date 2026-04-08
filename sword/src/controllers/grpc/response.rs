use tokio_stream::Stream;
use tonic::Status;

use super::GrpcStream;

pub struct GrpcResponse;

impl GrpcResponse {
    pub fn message<T>(value: T) -> tonic::Response<T> {
        tonic::Response::new(value)
    }

    pub fn stream<T, S>(stream: S) -> tonic::Response<GrpcStream<T>>
    where
        S: Stream<Item = Result<T, Status>> + Send + 'static,
    {
        tonic::Response::new(Box::pin(stream))
    }
}
