"""
Venice implements its own async runtime on top of Micropython. `vasyncio` provides primitives for working with async Python in Venice.
"""

from typing import Any

from . import TimeUnit

class EventLoop:
    """
    A cooperative scheduler for Venice coroutine tasks and timed sleeps.

    A *task* is a lightweight, non-blocking unit of execution. Tasks allow you to cooperatively perform
    work in the background without blocking other code from running.

    - Tasks are **lightweight**. Because tasks are scheduled and managed by Venice, creating new tasks
    or switching between tasks does not require a context switch and has fairly low overhead.
    Creating, running, and destroying large numbers of tasks is relatively cheap in comparison to
    traditional threads.
    - Tasks are scheduled **cooperatively**. A task will run until it voluntarily yields using an
    `await` point, giving control back to the event loop. The loop then switches to executing a
    different task.
    - Tasks are **non-blocking**. When a task cannot continue executing, it should yield, allowing the
    event loop to schedule another task in its place. Tasks should not perform operations that could
    block the CPU for a long period without an `await` point, because this prevents other tasks from
    executing as well. This includes long-running tight loops without `await` points.

    The loop runs one ready task at a time. Users normally call `vasyncio.run` instead of managing an
    event loop directly.
    """
    def __init__(self, /) -> None:
        """
        Creates an empty event loop.

        # Raises

        - `TypeError`: If any positional or keyword arguments are supplied.
        """
        ...

    def spawn(self, coro: Any, /) -> Task:
        """
        Schedules coroutine object `coro` on this loop and returns an awaitable `Task`.

        Scheduling does not run the coroutine until the loop is running. Awaiting the returned task
        yields its return value whether the task is still running or has already finished.

        # Raises

        - `TypeError`: If `coro` is not a coroutine object.
        """
        ...

    def run(self, /) -> None:
        """
        Runs scheduled tasks until no ready tasks or pending sleeps remain.

        While this method is running, `vasyncio.get_running_loop` returns this loop and
        `vasyncio.spawn` adds tasks to it. An exception raised by a task stops the loop and is
        propagated to the caller.

        # Raises

        - `RuntimeError`: If tasks form a direct or transitive await cycle.
        - `ValueError`: If a `Sleep` deadline is too large to represent.
        """
        ...

class Sleep:
    """
    An awaitable that will complete after a given duration.

    Awaiting a `Sleep` effectively yields the current task for a period of time. Constructing it does
    not block and does not start a separate task. When the duration has elapsed, awaiting it returns
    `None`.
    """
    def __init__(self, interval: float, unit: TimeUnit, /) -> None:
        """
        Waits until `interval`, measured in `unit`, has elapsed.

        This constructor returns an awaitable that will complete after the given duration, effectively
        yielding the current task for a period of time. Use `MILLIS` for milliseconds or `SECOND` for
        seconds. `interval` must be finite, non-negative, and small enough to represent.

        # Examples

        ```python
        from venice import *

        async def main():
        print("See you in 5 minutes.")
        await vasyncio.Sleep(300, SECOND)
        print("Hello again!")

        vasyncio.run(main())
        ```

        # Raises

        - `TypeError`: If any keyword argument is supplied.
        - `ValueError`: If `interval` is negative, non-finite, or too large to represent.
        """
        ...

class Task:
    """
    A spawned task.

    A `Task` can be awaited to retrieve the output of its coroutine.

    `EventLoop.spawn` and `vasyncio.spawn` return tasks; `Task` is not directly exported from the
    `vasyncio` submodule and is not constructed by users. Awaiting a task cooperatively waits for
    its coroutine and returns that coroutine's return value, including when the task completed
    before the await began. Direct or transitive cycles between awaited tasks raise `RuntimeError`.
    A coroutine exception propagates out of the running event loop. `Task` objects cannot be
    cancelled.

    # Examples

    ```python
    from venice import *

    async def work():
    print("Hello from a task!")
    return 1 + 2

    async def main():
    # Spawn a coroutine onto the event loop.
    task = vasyncio.spawn(work())

    # Wait for the task's output.
    assert await task == 3

    vasyncio.run(main())
    ```
    """

def run(coro: Any, /) -> None:
    """
    Runs coroutine object `coro` on a new event loop until no work remains.

    The loop also waits for tasks spawned into it and pending `Sleep` objects before returning
    `None`. The root coroutine's return value is discarded, and an exception from any task stops
    the loop and is propagated to the caller.

    # Examples

    ```python
    from venice import *

    async def main():
    print("start")
    await vasyncio.Sleep(100, MILLIS)
    print("done")

    vasyncio.run(main())
    ```

    # Raises

    - `TypeError`: If `coro` is not a coroutine object.
    - `RuntimeError`: If tasks form a direct or transitive await cycle.
    - `ValueError`: If a `Sleep` deadline is too large to represent.
    """
    ...

def spawn(coro: Any, /) -> Task:
    """
    Spawns a new asynchronous task that can be controlled with the returned `Task` handle.

    Call this from code already executing under `vasyncio.run` or `EventLoop.run`. Awaiting the task
    yields the coroutine's return value whether the task is still running or has already completed.

    # Examples

    ```python
    from venice import *

    async def answer():
    await vasyncio.Sleep(10, MILLIS)
    return 42

    async def main():
    task = vasyncio.spawn(answer())
    print(await task)

    vasyncio.run(main())
    ```

    # Raises

    - `RuntimeError`: If no event loop is running.
    - `TypeError`: If `coro` is not a coroutine object.
    """
    ...

def get_running_loop() -> EventLoop | None:
    """
    Returns the event loop currently executing tasks, or `None` outside `vasyncio.run` or
    `EventLoop.run`.
    """
    ...
