"""
Display and touchscreen input.

Contains user calls to the V5 Brain display for touching and displaying graphics.
"""

from typing import Any, ClassVar

from . import Color

class RenderMode:
    """
    The rendering mode for the VEX V5's display, available as `display.RenderMode`.

    When using the display in `RenderMode.IMMEDIATE` mode, all draw operations will immediately show
    up on the display. `RenderMode.DOUBLE_BUFFERED` mode instead applies draw operations onto an
    intermediate buffer that can be swapped onto the display by calling `display.render`, thereby
    preventing screen tearing. By default, the display uses `RenderMode.IMMEDIATE` mode.

    # Note

    `display.render` **MUST** be called for anything to appear on the display when using
    `RenderMode.DOUBLE_BUFFERED` mode. Values print as their qualified constant names.
    """

    IMMEDIATE: ClassVar[RenderMode]
    """Draw operations are immediately applied to the display without the need to call `display.render`."""
    DOUBLE_BUFFERED: ClassVar[RenderMode]
    """
    Draw calls are affected on an intermediary display buffer, rather than directly drawn to the
    display. The intermediate buffer can later be applied to the display using `display.render`.

    This mode is necessary for preventing screen tearing when drawing at high speeds.
    """

class FontFamily:
    """The font family used by `display.draw_text`, available as `display.FontFamily`."""

    MONOSPACE: ClassVar[FontFamily]
    """
    A monospaced font which has a fixed width for each character.

    This font at full size is 49pt Noto Mono.
    """
    PROPORTIONAL: ClassVar[FontFamily]
    """
    A proportional font which has a varying width for each character.

    This font at full size is 49pt Noto Sans.
    """

class FontSize:
    """
    A fractional font scaling factor, available as `display.FontSize`.

    The read-only `numerator` attribute is the numerator of the fractional font scale. The read-only
    `denominator` attribute is the denominator of the fractional font scale. Use one of the predefined
    constants or construct a custom nonnegative fraction with a nonzero denominator. Values print as
    `FontSize(numerator=..., denominator=...)`.
    """

    numerator: int
    denominator: int
    EXTRA_SMALL: ClassVar[FontSize]
    """An extra-small font size with a value of one-fifth."""
    SMALL: ClassVar[FontSize]
    """A small font size with a value of one-fourth."""
    MEDIUM: ClassVar[FontSize]
    """A medium font size with a value of one-third."""
    LARGE: ClassVar[FontSize]
    """A large font size with a value of one-half."""
    EXTRA_LARGE: ClassVar[FontSize]
    """An extra-large font size with a value of two-thirds."""
    FULL: ClassVar[FontSize]
    """The full size of the font."""

    def __init__(self, numerator: int, denominator: int, /) -> None:
        """
        Creates a custom fractional font size from `numerator` and `denominator`.

        Both values must be nonnegative integers, and `denominator` must be greater than zero.

        # Raises

        - `ValueError`: If `numerator` or `denominator` is negative or outside the supported integer
        range, or if `denominator` is zero.
        """
        ...

class TouchEvent:
    """
    A read-only touch event on the display, available as `display.TouchEvent`.

    - `x` and `y` are the point at which the display was touched, in pixels from the top-left.
    - `press_count` is the number of times the display has been pressed.
    - `release_count` is the number of times the display has been released.
    - `is_now_pressed` is `True` if the touch has just been pressed.
    - `is_pressed` is `True` if the display has been touched or is still being held.
    - `is_released` is `True` if the touch has been released.
    - `is_held` is `True` if the display has been touched and is still being held.

    Snapshots compare by value, return `False` when compared with another type, and print as
    `TouchEvent(x=..., y=..., press_count=..., release_count=..., is_now_pressed=...,
    is_pressed=..., is_released=..., is_held=...)`. They are obtained from
    `display.get_touch_status`, not constructed directly.
    """

    x: int
    y: int
    press_count: int
    release_count: int
    is_now_pressed: bool
    is_pressed: bool
    is_released: bool
    is_held: bool

def draw_pixel(x: int, y: int, color: Color) -> None:
    """
    Draws a filled pixel to the display with the specified `color`.

    `x` and `y` are pixel coordinates. The writable display is 480 pixels wide by 240 pixels high, with
    its origin at the top-left.
    """
    ...

def draw_line(
    start_x: int, start_y: int, end_x: int, end_y: int, color: Color, /
) -> None:
    """
    Draws a line to the display with the specified `color`.

    `start_x` and `start_y` are the start point of the line; `end_x` and `end_y` are the end point. The
    line width is one pixel. Coordinates are measured from the display's top-left, and both supplied
    endpoints are passed directly to the display SDK.
    """
    ...

def draw_circle(x: int, y: int, radius: int, color: Color, /) -> None:
    """
    Draws a outlined circle to the display with the specified `color`.

    `x` and `y` are the center point of the circle, and `radius` is its radius in pixels. Circles are
    not antialiased.
    """
    ...

def fill_circle(x: int, y: int, radius: int, color: Color, /) -> None:
    """
    Draws a filled circle to the display with the specified `color`.

    `x` and `y` are the center point of the circle, and `radius` is its radius in pixels. Circles are
    not antialiased.
    """
    ...

def draw_rect(x: int, y: int, width: int, height: int, color: Color, /) -> None:
    """
    Draws an outlined rectangular region of the display with the specified `color`.

    `x` and `y` are the top-left coordinate of the rectangle. `width` and `height` are its dimensions
    in pixels. The bottom right point is not included in the shape's bounds. Thus, the area of the
    drawn rectangle is `width * height` pixels.

    # Examples

    ```python
    from venice import *

    # Draw a 20x20 rectangle which has a top-left point at (30, 40).
    display.draw_rect(30, 40, 20, 20, Color.WHITE)
    ```

    # Raises

    - `ValueError`: If a coordinate, dimension, or resulting endpoint is outside its supported
    integer range.
    """
    ...

def fill_rect(x: int, y: int, width: int, height: int, color: Color, /) -> None:
    """
    Draws a filled rectangular region of the display with the specified `color`.

    `x` and `y` are the top-left coordinate of the rectangle. `width` and `height` are its dimensions
    in pixels. The bottom right point is not included in the shape's bounds. Thus, the area of the
    drawn rectangle is `width * height` pixels.

    # Examples

    ```python
    from venice import *

    # Draw a 20x20 rectangle which has a top-left point at (30, 40).
    display.fill_rect(30, 40, 20, 20, Color.WHITE)
    ```

    # Raises

    - `ValueError`: If a coordinate, dimension, or resulting endpoint is outside its supported
    integer range.
    """
    ...

def draw_buffer(x: int, y: int, width: int, height: int, buffer: Any, /) -> None:
    """
    Draws a buffer of pixels to a specified region of the display.

    This function copies the pixels in `buffer` to the specified region of the display. `x` and `y`
    are the region's top-left corner, and `width` and `height` are measured in pixels. `buffer` must be
    a readable, suitably aligned buffer containing exactly `width * height` packed four-byte
    `0xRRGGBB` color values in row-major order.

    # Raises

    - `TypeError`: If `buffer` doesn't support the readable buffer protocol.
    - `ValueError`: If the buffer is unaligned, its readable byte length isn't a multiple of four,
    its pixel count doesn't equal `width * height`, or the region exceeds the supported coordinate
    range.
    """
    ...

def draw_text(
    text: str,
    x: int,
    y: int,
    /,
    *,
    font_size: FontSize = FontSize.MEDIUM,
    font_family: FontFamily = FontFamily.PROPORTIONAL,
    color: Color = Color.WHITE,
    bg_color: Color | None = None,
) -> None:
    """
    Draws a line of `text` with the specified `color` and `bg_color` to the display.

    `x` and `y` are the top-left corner coordinates of the text. `font_size` defaults to
    `FontSize.MEDIUM`, `font_family` defaults to `FontFamily.PROPORTIONAL`, and `color` defaults to
    `Color.WHITE`. Omitting `bg_color` or passing it as `None` gives a transparent background;
    supply a `Color` to paint it.

    # Examples

    ```python
    from venice import *

    # Write red text with a blue background to the display.
    display.draw_text(
    "Hello, World!",
    10,
    10,
    font_size=display.FontSize.MEDIUM,
    font_family=display.FontFamily.MONOSPACE,
    color=Color(255, 0, 0),
    bg_color=Color(0, 0, 255),
    )
    ```

    # Raises

    - `ValueError`: If `text` contains a NUL character.
    """
    ...

def print(*values: object, sep: str = " ", end: str = "\n") -> None:
    """
    Writes `values` to the Brain display's scrolling text area.

    Values are converted to strings and joined with `sep`, which defaults to one space, then `end` is
    appended, which defaults to a newline. Text uses the default white font, wraps after 48 characters,
    and scrolls upward after 12 visible lines.

    # Raises

    - `TypeError`: If a value isn't printable or if `sep` or `end` isn't a string.
    """
    ...

def scroll(start: int, offset: int) -> None:
    """
    Scrolls the pixels at or below the specified y-coordinate `start`.

    This function y-offsets the pixels in the display buffer which are at or below `start` by `offset`
    pixels. Positive values move the pixels upwards, and pixels that are moved out of the scroll region
    are discarded. Empty spaces are then filled with the display's background color.
    """
    ...

def scroll_region(x: int, y: int, width: int, height: int, offset: int, /) -> None:
    """
    Scrolls a region of the display.

    This function y-offsets the pixels in the display buffer which are contained in the specified
    scroll region by `offset` pixels. `x` and `y` are the region's top-left corner; `width` and `height`
    are measured in pixels. Positive offset values move the pixels upwards, and pixels that are moved
    out of the scroll region are discarded. Empty spaces are then filled with the display's background
    color.

    # Raises

    - `ValueError`: If `x`, `y`, or `offset` is outside the supported signed 16-bit range, `width` or
    `height` is outside the unsigned 16-bit range, or the region's endpoint exceeds the supported
    coordinate range.
    """
    ...

def set_render_mode(render_mode: RenderMode) -> None:
    """
    Sets `render_mode` for the display.

    For more information on render modes, see `display.RenderMode`.
    """
    ...

def render() -> None:
    """
    Flushes the display's double buffer if it is enabled.

    This is a no-op with `RenderMode.IMMEDIATE`, but is necessary for anything to be displayed when
    using `RenderMode.DOUBLE_BUFFERED`.
    """
    ...

def erase(color: Color) -> None:
    """Clears the entire 480-by-240-pixel writable display, filling it with the specified `color`."""
    ...

def get_touch_status() -> TouchEvent:
    """
    Returns the last recorded state of the display's touchscreen as a `TouchEvent`.

    See `display.TouchEvent` for more information.
    """
    ...

def is_now_pressed() -> bool:
    """Returns whether the touchscreen's last recorded touch has just been pressed."""
    ...

def is_pressed() -> bool:
    """Returns whether the touchscreen's last recorded touch has been pressed or is being held."""
    ...

def is_released() -> bool:
    """Returns whether the touchscreen's last recorded touch has been released."""
    ...

def is_held() -> bool:
    """Returns whether the touchscreen's last recorded touch is still being held."""
    ...
