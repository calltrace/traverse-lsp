use crate::utils::TOKIO_RUNTIME;
use anyhow::Result;
use std::sync::mpsc;
use tokio::sync::oneshot;

pub fn send_request_to_worker<TRequest, TResponse>(
    tx: &mpsc::Sender<TRequest>,
    build_request: impl FnOnce(oneshot::Sender<TResponse>) -> TRequest,
) -> Result<TResponse, mpsc::SendError<TRequest>> {
    let (response_tx, response_rx) = oneshot::channel();
    let request = build_request(response_tx);
    tx.send(request)?;
    Ok(TOKIO_RUNTIME.block_on(response_rx).unwrap())
}
