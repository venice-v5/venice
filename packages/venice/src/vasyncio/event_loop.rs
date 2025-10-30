use std::{
    cell::RefCell,
    collections::{binary_heap::BinaryHeap, vec_deque::VecDeque},
    time::Instant,
};

use micropython_rs::{
    const_dict,
    except::raise_type_error,
    fun::{Fun1, Fun2},
    generator::{VmReturnKind, mp_type_gen_instance, resume_gen},
    init::token,
    map::Dict,
    nlr::raise,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk::vexTasksRun;

use super::task::Task;
use crate::{obj::alloc_obj, qstrgen::qstr, vasyncio::sleep::Sleep};

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
    deadline: Instant,
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
    const OBJ_TYPE: *const ObjType = EVENT_LOOP_TYPE.as_obj_type_ptr();
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

    // returns:
    // true -> no more tasks/sleepers to run, stop
    // false -> tasks/sleepers still in queues
    pub fn tick(&self) -> bool {
        let mut ready = self.ready.borrow_mut();
        let mut sleepers = self.sleepers.borrow_mut();

        if let Some(sleeper) = sleepers.peek()
            && sleeper.deadline <= Instant::now()
        {
            let sleeper = sleepers.pop().unwrap();
            ready.push_back(sleeper.task);
        }

        if let Some(obj) = ready.pop_front() {
            let task = obj.as_obj::<Task>().unwrap();
            let result = resume_gen(task.coro(), Obj::NONE, Obj::NULL);
            match result.return_kind {
                VmReturnKind::Normal => {}
                VmReturnKind::Yield => {
                    if let Some(sleep) = result.obj.as_obj::<Sleep>() {
                        sleepers.push(Sleeper {
                            task: obj,
                            deadline: sleep.deadline(),
                        });
                    }
                }
                VmReturnKind::Exception => raise(token().unwrap(), result.obj),
            }
        }

        unsafe { vexTasksRun() };
        sleepers.is_empty() && ready.is_empty()
    }

    pub fn run(&self) {
        while !self.tick() {}
    }
}

pub extern "C" fn new_event_loop() -> Obj {
    alloc_obj(EventLoop::new())
}

extern "C" fn event_loop_spawn(self_in: Obj, coro: Obj) -> Obj {
    if !coro.is(&raw const mp_type_gen_instance) {
        raise_type_error(token().unwrap(), "expected coroutine");
    }

    self_in.as_obj::<EventLoop>().unwrap().spawn(coro)
}

pub extern "C" fn event_loop_run(self_in: Obj) -> Obj {
    self_in.as_obj::<EventLoop>().unwrap().run();
    Obj::NULL
}
