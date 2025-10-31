use std::{
    cell::RefCell,
    collections::{binary_heap::BinaryHeap, vec_deque::VecDeque},
    sync::Mutex,
    time::Instant,
};

use micropython_rs::{
    const_dict,
    except::{mp_type_RuntimeError, raise_msg, raise_type_error},
    fun::{Fun1, Fun2},
    generator::{VmReturnKind, mp_type_gen_instance, resume_gen},
    init::token,
    nlr,
    nlr::push_nlr_callback,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk::vexTasksRun;

use super::task::Task;
use crate::{obj::alloc_obj, qstrgen::qstr, vasyncio::sleep::Sleep};

pub static EVENT_LOOP_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(EventLoop))
        .set_slot_make_new(event_loop_new)
        .set_slot_locals_dict_from_static({
            &const_dict![
                qstr!(spawn) => Obj::from_static(&Fun2::new(event_loop_spawn)),
                qstr!(run) => Obj::from_static(&Fun1::new(event_loop_run)),
            ]
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
    const OBJ_TYPE: *const ObjType = EVENT_LOOP_OBJ_TYPE.as_obj_type_ptr();
}

static CURRENT_EVENT_LOOP: Mutex<Obj> = Mutex::new(Obj::NONE);

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

        let task_obj = ready.pop_front();
        // let the task access the event loop while it's running
        drop(ready);
        drop(sleepers);

        if let Some(task_obj) = task_obj {
            let task = task_obj.as_obj::<Task>().unwrap();

            let result = resume_gen(task.coro(), Obj::NONE, Obj::NULL);
            match result.return_kind {
                VmReturnKind::Normal => {
                    let mut ready = self.ready.borrow_mut();
                    task.complete_with(result.obj);
                    task.waiting_tasks()
                        .iter()
                        .for_each(|&w| ready.push_front(w));
                }
                VmReturnKind::Yield => {
                    if let Some(sleep) = result.obj.as_obj::<Sleep>() {
                        self.sleepers.borrow_mut().push(Sleeper {
                            task: task_obj,
                            deadline: sleep.deadline(),
                        });
                    } else if let Some(awaited_task) = result.obj.as_obj::<Task>() {
                        awaited_task.add_waiting_task(task_obj);
                    }
                }
                VmReturnKind::Exception => nlr::raise(token().unwrap(), result.obj),
            }
        }

        unsafe { vexTasksRun() };
        self.sleepers.borrow().is_empty() && self.ready.borrow().is_empty()
    }

    pub fn run(&self) {
        while !self.tick() {}
    }
}

extern "C" fn event_loop_new(_: *const ObjType, n_args: usize, n_kw: usize, _: *const Obj) -> Obj {
    if n_args != 0 || n_kw != 0 {
        raise_type_error(token().unwrap(), "function does not accept any arguments");
    }

    alloc_obj(EventLoop::new())
}

extern "C" fn event_loop_spawn(self_in: Obj, coro: Obj) -> Obj {
    if !coro.is(&raw const mp_type_gen_instance) {
        raise_type_error(token().unwrap(), "expected coroutine");
    }

    self_in.as_obj::<EventLoop>().unwrap().spawn(coro)
}

extern "C" fn event_loop_run(self_in: Obj) -> Obj {
    *CURRENT_EVENT_LOOP.lock().unwrap() = self_in;
    push_nlr_callback(
        token().unwrap(),
        || self_in.as_obj::<EventLoop>().unwrap().run(),
        || *CURRENT_EVENT_LOOP.lock().unwrap() = Obj::NONE,
        true,
    );
    Obj::NONE
}

pub extern "C" fn get_running_loop() -> Obj {
    *CURRENT_EVENT_LOOP.lock().unwrap()
}

pub extern "C" fn vasyncio_run(coro: Obj) -> Obj {
    if !coro.is(&raw const mp_type_gen_instance) {
        raise_type_error(token().unwrap(), "expected coroutine");
    }

    let eloop = EventLoop::new();
    eloop.spawn(coro);
    event_loop_run(alloc_obj(eloop))
}

pub extern "C" fn vasyncio_spawn(coro: Obj) -> Obj {
    let eloop = get_running_loop();
    if eloop.is_none() {
        raise_msg(
            token().unwrap(),
            &raw const mp_type_RuntimeError,
            "no running event loop",
        );
    }

    event_loop_spawn(eloop, coro)
}
