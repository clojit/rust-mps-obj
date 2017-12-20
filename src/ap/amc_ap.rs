


#[derive(Clone)]
pub struct structAp {
    raw: Arc<RawArena>,
}

impl structAp {
    /// Creates a new virtual memory arena with the specified initial size
    pub fn with_pool(capacity: usize) -> Result<Self> {
        let args = mps_args! {
             MPS_KEY_ARENA_SIZE: capacity,
        };

        let arena = unsafe {
            let mut arena: mps_arena_t = ptr::null_mut();
            let res = mps_arena_create_k(&mut arena, mps_arena_class_vm(), args);

            Error::result(res).map(|_| RawArena { arena })
        }?;

        Ok(structAp {
            raw: Arc::new(arena),
        })
    }
}

impl Arena for structAp {
    fn as_raw(&self) -> mps_arena_t {
        self.raw.arena
    }
}

impl ArenaRef for structAp {
    fn acquire(&self) -> Self {
        self.clone()
    }
}
