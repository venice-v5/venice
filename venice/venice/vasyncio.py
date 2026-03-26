"""
Venice implements its own async runtime on top of Micropython. `vasyncio` provides primitives for working with async Python in Venice.
"""
from typing import TypeVar, Generic, Awaitable, Any
from . import TimeUnit

_T = TypeVar('_T')

class Task(Generic[_T]):
    """A scheduled asynchronous task.

    Tasks are created by `EventLoop.spawn` or `spawn`. They wrap a coroutine that will be resumed
    by the event loop until it completes, at which point awaiting the task yields the coroutine's
    return value.
    """

    def __iter__(self) -> 'Task[_T]':
        """Await this task until it completes.

        When the task finishes, awaiting it produces the coroutine's return value.
        """
        ...


class EventLoop:
    """A cooperative event loop for Venice coroutines.

    The event loop maintains a queue of ready tasks and a queue of sleeping tasks. Calling `run`
    repeatedly advances scheduled coroutines until there is no more work left to do.
    """

    def __init__(self) -> None:
        """Create a new empty event loop."""
        ...

    def spawn(self, coro: Awaitable[_T]) -> Task[_T]:
        """Schedule a coroutine on this event loop and return its task handle."""
        ...

    def run(self) -> None:
        """Run this event loop until there are no ready or sleeping tasks remaining."""
        ...


class Sleep:
    """An awaitable sleep operation.

    Instances of this class yield control back to the Venice event loop until the requested
    duration has elapsed.
    """

    def __init__(self, interval: float, unit: TimeUnit) -> None:
        """Create a sleep operation for the given duration and time unit."""
        ...

    def __iter__(self) -> 'Sleep':
        """Await this sleep operation until it completes."""
        ...


def get_running_loop() -> EventLoop | None:
    """Return the currently running event loop.

    If no event loop is currently running, this returns `None`.
    """
    ...


def run(coro: Awaitable[Any]) -> None:
    """Create a fresh event loop, schedule `coro`, and run it to completion.

    This function installs the new loop as the running loop for the duration of the call.
    """
    ...


def spawn(coro: Awaitable[_T]) -> Task[_T]:
    """Schedule `coro` on the currently running event loop.

    Raises a runtime error if called when no event loop is running.
    """
    ...
