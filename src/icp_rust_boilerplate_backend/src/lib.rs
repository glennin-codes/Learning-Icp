use serde::{ Deserialize, Serialize };
use candid::{ CandidType, Decode, Encode };
use ic_cdk::api::time;
use ic_stable_structures::{
    memory_manager::{ MemoryId, MemoryManager, VirtualMemory },
    BoundedStorable,
    Storable,
    StableBTreeMap,
    Cell,
    DefaultMemoryImpl,
};
use std::{ borrow::Cow, cell::RefCell, fmt };

// Defining Memory and IdCell
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// The IdCell is a cell responsible for holding the current ID of the message. We'll utilize this to generate unique IDs for each message.

// Defining Message Struct

#[derive(Debug, CandidType, Clone, Serialize, Deserialize, Default)]
struct Message {
    id: u64,
    title: String,
    body: String,
    attachement_url: String,
    created_at: u64,
    updated_at: Option<u64>,
}
#[derive(Serialize, Deserialize, CandidType)]
struct ApiResponse<T> {
    data: Option<T>,
    logs: Vec<String>,
    error: Option<MyError>,
}
#[derive(Debug, candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct MessagePayload {
    title: String,
    body: String,
    attachment_url: String,
}
// Implementing Storable and BoundedStorable

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Message {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
// another trait that must be implemented for a struct that is stored in a stable struct

impl BoundedStorable for Message {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
// The Storable trait is used to convert a struct to bytes and vice versa. The BoundedStorable trait is used to define the maximum size of a struct and whether it is a fixed size or not.

//Setting up Thread Local Variables
//Thread-local variable that will hold our canisters state.
//Thread-local variables are variables that are local to the current threaf (sequence of instructions).They are useful when you need to share data between multiple threads.

//we will use a 'Refcell' to manage our canister's state allowing us to access it from anwhere in our code.A 'Refcell' is a mutable memory location with dynamically checked borrow rules .It is usefull when your confident that your code atheres to borrowing rules ,but the compiler cannot guarantee that.

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0
        ).expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Message, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );
}

//custom error handler
#[derive(Debug, CandidType, Serialize, Deserialize)]
enum MyError {
    NotFound {
        msg: String,
    },
    SerializationError {
        msg: String,
    },
}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::NotFound { msg } => write!(f, "{}", msg),
            MyError::SerializationError { msg } => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MyError {}

#[ic_cdk::query]
fn get_message(id: u64) -> Result<Message, MyError> {
    println!("id :{}", id);
    match _get_message(&id) {
        Some(message) => {
            println!("message:{:#?}", message);
            Ok(message)
        }
        None =>
            Err(MyError::NotFound {
                msg: format!("a message with id={} not fount", id),
            }),
    }
}
fn _get_message(id: &u64) -> Option<Message> {
    STORAGE.with(
        |service| 
        service.borrow().get(id)
    )
}
#[ic_cdk::query]
fn get_all_messages() -> ApiResponse<Vec<Message>> {
    STORAGE.with(|storage| {
        let borrowed_messages = storage.borrow();
        let messages:Vec<Message> = borrowed_messages
            .iter()
            .map(|(_,message)| message)
            .collect();
      println!("json{:#?}",messages);
                
                ApiResponse {
                    data: Some(messages),
                    logs: vec![String::from("Succesfully Retrieved all messages")],
                    error: None,
                }
            
        
    })
}

// do_insert Function
#[ic_cdk::update]
fn add_message(message: MessagePayload) -> Option<Message> {
    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    }).expect("cannot increment id counter");
    let message = Message {
        id,
        title: message.title,
        body: message.body,
        attachement_url: message.attachment_url,
        created_at: time(),
        updated_at: None,
    };
    insert_message(&message);
    Some(message)
}

//update_message Function
#[ic_cdk::update]
fn update_message(id: u64, payload: MessagePayload) -> Result<Message, MyError> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut message) => {
            message.attachement_url = payload.attachment_url;
            message.body = payload.body;
            message.title = payload.title;
            message.updated_at = Some(time());
            insert_message(&message);
            Ok(message)
        }
        None =>
            Err(MyError::NotFound {
                msg: format!("couldn't update a message with id={}. message not found", id),
            }),
    }
}
#[ic_cdk::update]
//delete_message Function
fn delete_message(id: u64) -> Result<Message, MyError> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(message) => Ok(message),
        None =>
            Err(MyError::NotFound {
                msg: format!("couldn't delete message of id={}. message not found", id),
            }),
    }
}

//helper function to insert message

fn insert_message(message: &Message) {
    STORAGE.with(|service| { service.borrow_mut().insert(message.id, message.clone()) });
}

//To generate the candid interface for each function
ic_cdk::export_candid!();
