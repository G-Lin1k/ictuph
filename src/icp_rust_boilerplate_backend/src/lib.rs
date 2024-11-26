#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Message {
    id: u64,
    title: String,
    body: String,
    attachment_url: String,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct MessagePayload {
    title: String,
    body: String,
    attachment_url: String,
}

impl BoundedStorable for Message {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)).unwrap()))
    );
    static STORAGE: RefCell<StableBTreeMap<u64, Message, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)).unwrap())
        )
    );
}

#[ic_cdk::query]
fn get_message(id: u64) -> Result<Message, Error> {
    STORAGE.with(|s| s.borrow().get(&id)).ok_or(Error::NotFound)
}

#[ic_cdk::update]
fn add_message(payload: MessagePayload) -> Option<Message> {
    let id = ID_COUNTER.with(|counter| counter.borrow_mut().increment());
    let message = Message {
        id,
        title: payload.title,
        body: payload.body,
        attachment_url: payload.attachment_url,
        created_at: time(),
        updated_at: None,
    };
    STORAGE.with(|s| s.borrow_mut().insert(id, message.clone()));
    Some(message)
}

#[ic_cdk::update]
fn update_message(id: u64, payload: MessagePayload) -> Result<Message, Error> {
    STORAGE.with(|s| {
        let mut msg = s.borrow().get(&id).ok_or(Error::NotFound)?;
        msg.title = payload.title;
        msg.body = payload.body;
        msg.attachment_url = payload.attachment_url;
        msg.updated_at = Some(time());
        s.borrow_mut().insert(id, msg.clone());
        Ok(msg)
    })
}

#[ic_cdk::update]
fn delete_message(id: u64) -> Result<Message, Error> {
    STORAGE.with(|s| s.borrow_mut().remove(&id)).ok_or(Error::NotFound)
}

#[derive(candid::CandidType, Serialize, Deserialize)]
enum Error {
    NotFound { msg: String },
}

impl Storable for Message {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

static ID_COUNTER: RefCell<IdCell> = RefCell::new(
    IdCell::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)).unwrap()),
        0, // Default value for the ID counter
    )
    .expect("Failed to initialize ID_COUNTER")
);
