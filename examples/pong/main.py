from game import Game
from render import configure_display, draw
from venice import MILLIS, Controller, monotonic_time, vasyncio

FRAME_MS = 16
INPUT_MS = Controller.UPDATE_INTERVAL_MS
DEAD_ZONE = 0.18


class Controls:
    def __init__(self):
        self.left = 0.0
        self.right = 0.0
        self.pause_requested = False
        self.restart_requested = False


def with_dead_zone(value):
    return value if abs(value) >= DEAD_ZONE else 0.0


async def read_controls(controller, controls):
    while True:
        state = controller.read_state()
        controls.left = -with_dead_zone(state.left_stick.y)
        controls.right = -with_dead_zone(state.right_stick.y)
        controls.pause_requested |= state.button_a.is_now_pressed
        controls.restart_requested |= state.button_x.is_now_pressed
        await vasyncio.Sleep(INPUT_MS, MILLIS)


async def run_game(game, controls):
    configure_display()
    previous_ms = monotonic_time(MILLIS)

    while True:
        now_ms = monotonic_time(MILLIS)
        elapsed = min((now_ms - previous_ms) / 1000, 0.05)
        previous_ms = now_ms

        if controls.restart_requested:
            controls.restart_requested = False
            game.reset_match()
        if controls.pause_requested:
            controls.pause_requested = False
            game.paused = not game.paused

        game.move_paddles(controls.left, controls.right, elapsed)
        game.update(elapsed)

        draw(game)
        await vasyncio.Sleep(FRAME_MS, MILLIS)


async def main():
    game = Game()
    controls = Controls()
    controller = Controller()

    # The event loop keeps spawned tasks alive alongside this long-running game loop.
    vasyncio.spawn(read_controls(controller, controls))
    await run_game(game, controls)


vasyncio.run(main())
