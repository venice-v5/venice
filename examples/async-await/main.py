from venice import vasyncio
import venice

async def sleep(ms):
    await vasyncio.Sleep(ms, venice.TimeUnit.MILLIS)
    print(f"slept {ms} ms")
    return 100

async def main():
    print("Starting...")
    task = vasyncio.spawn(sleep(500))
    await sleep(5000)
    ret = await task
    print(f"task returned {ret}")

vasyncio.run(main())
