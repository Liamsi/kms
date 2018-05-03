//! A session with a validator node

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use error::Error;
use rpc::{Request, Response, SignRequest, SignResponse};
use ed25519::{Keyring, PublicKey};

/// A (soon-to-be-encrypted) session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    socket: TcpStream,

    /// Keyring of signature keys
    keyring: Arc<Keyring>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(addr: &str, port: u16, keyring: Arc<Keyring>) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let mut socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        Ok(Self { socket, keyring })
    }

    /// Handle an incoming request from the validator
    pub fn handle_request(&mut self) -> Result<(), Error> {
        let response = match Request::read(&mut self.socket)? {
            Request::Sign(ref req) => self.sign(req)?,
        };

        self.socket.write_all(&response.to_vec())?;
        Ok(())
    }

    /// Perform a digital signature operation
    fn sign(&mut self, request: &SignRequest) -> Result<Response, Error> {
        let pk = PublicKey::from_bytes(&request.public_key)?;
        let signature = self.keyring.sign(&pk, &request.msg)?;

        Ok(Response::Sign(SignResponse {
            sig: signature.as_bytes().to_vec(),
        }))
    }
}
