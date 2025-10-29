use std::{
    cell::RefCell,
    collections::{binary_heap::BinaryHeap, vec_deque::VecDeque},
};

use micropython_rs::{
    const_dict,
    fun::{Fun1, Fun2},
    generator::{VmReturnKind, resume_gen},
    init::token,
    map::Dict,
    nlr::raise,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk::vexTasksRun;

use super::task::Task;
use crate::{obj::alloc_obj, qstrgen::qstr};

pub static EVENT_LOOP_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(EventLoop))
    .set_slot_locals_dict({
        static mut LOCALS_DICT: Dict = const_dict![
            qstr!(spawn) => Fun2::new(event_loop_spawn).as_obj(),
            qstr!(run) => Fun1::new(event_loop_run).as_obj(),
        ];
        &raw mut LOCALS_DICT
    });

struct Sleeper {
    task: Obj,
    // TODO: replace with Instant
    deadline: u32,
}

impl PartialEq for Sleeper {
    fn eq(&self, other: &Self) -> bool {
        self.deadline.eq(&other.deadline)
    }
}

impl Eq for Sleeper {}

impl PartialOrd for Sleeper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sleeper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deadline.cmp(&other.deadline).reverse()
    }
}

#[repr(C)]
pub struct EventLoop {
    base: ObjBase,
    ready: RefCell<VecDeque<Obj>>,
    sleepers: RefCell<BinaryHeap<Sleeper>>,
}

unsafe impl ObjTrait for EventLoop {
    const OBJ_TYPE: *const ObjType = &raw const EVENT_LOOP_TYPE as *const ObjType;
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            base: ObjBase::new::<EventLoop>(),
            ready: RefCell::new(VecDeque::new()),
            sleepers: RefCell::new(BinaryHeap::new()),
        }
    }

    pub fn spawn(&self, coro: Obj) -> Obj {
        let task = alloc_obj(Task::new(coro));
        self.ready.borrow_mut().push_back(task);
        task
    }

    pub fn tick(&self) -> bool {
        let mut ready = self.ready.borrow_mut();
        let mut sleepers = self.sleepers.borrow_mut();

        if let Some(sleeper) = sleepers.peek()
            && sleeper.deadline <= u32::MAX
        {
            let sleeper = sleepers.pop().unwrap();
            ready.push_back(sleeper.task);
        }

        let ret = if let Some(obj) = ready.pop_front() {
            let task = obj.as_obj::<Task>().unwrap();
            let result = resume_gen(task.coro(), Obj::NONE, Obj::NULL);
            match result.return_kind {
                VmReturnKind::Normal => {}
                VmReturnKind::Yield => ready.push_back(obj),
                VmReturnKind::Exception => raise(token().unwrap(), result.obj),
            }
            true
        } else {
            false
        };

        unsafe { vexTasksRun() };
        ret
    }

    pub fn run(&self) {
        while self.tick() {}
    }
}

pub extern "C" fn new_event_loop() -> Obj {
    alloc_obj(EventLoop::new())
}

pub extern "C" fn event_loop_spawn(self_in: Obj, coro: Obj) -> Obj {
    self_in.as_obj::<EventLoop>().unwrap().spawn(coro)
}

pub extern "C" fn event_loop_run(self_in: Obj) -> Obj {
    self_in.as_obj::<EventLoop>().unwrap().run();
    Obj::NULL
}
