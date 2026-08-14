import venice
from venice import vasyncio

BUTTONS = (
    ("A", "button_a"),
    ("B", "button_b"),
    ("X", "button_x"),
    ("Y", "button_y"),
    ("Up", "button_up"),
    ("Down", "button_down"),
    ("Left", "button_left"),
    ("Right", "button_right"),
    ("L1", "button_l1"),
    ("L2", "button_l2"),
    ("R1", "button_r1"),
    ("R2", "button_r2"),
)


async def log_joysticks(controller):
    while True:
        state = controller.read_state()
        print(
            f"Joysticks: left=({state.left_stick.x}, {state.left_stick.y}), "
            f"right=({state.right_stick.x}, {state.right_stick.y})"
        )
        await vasyncio.Sleep(500, venice.TimeUnit.MILLIS)


async def main():
    await vasyncio.Sleep(500, venice.TimeUnit.MILLIS)
    controller = venice.Controller()
    vasyncio.spawn(log_joysticks(controller))

    while True:
        state = controller.read_state()

        for name, attribute in BUTTONS:
            if getattr(state, attribute).is_now_pressed:
                print(f"Button {name} pressed")

        await vasyncio.Sleep(controller.UPDATE_INTERVAL_MS, venice.TimeUnit.MILLIS)


vasyncio.run(main())
