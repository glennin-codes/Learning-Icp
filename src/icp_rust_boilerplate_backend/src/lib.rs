use serde;
use candid::{ Decode, Encode };
use ic_cdk::api::time;
use ic_stable_structures::{
    memory_manager::{ MemoryId, MemoryManager, VirtualMemory },
    Cell,
    DefaultMemoryImpl,
};
use std::{ borrow::Cow, cell::RefCell };

// Defining Memory and IdCell
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// The IdCell is a cell responsible for holding the current ID of the message. We'll utilize this to generate unique IDs for each message.
