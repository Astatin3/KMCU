pub(crate) mod vlq;

pub(crate) mod types {
    pub mod command;
    pub mod dictionary;
    pub mod message;
    pub mod serial;
}

pub(crate) mod traits {
    pub mod binary;
    pub mod connection;
}

pub mod connections;

pub use traits::connection::Connection;
