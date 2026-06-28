"""
Display and touchscreen input.

Contains user calls to the V5 Brain display for touching and displaying graphics.
"""
from __future__ import annotations
from typing import ClassVar, Any
from . import Color

class RenderMode:
    IMMEDIATE: ClassVar[RenderMode]
    DOUBLE_BUFFERED: ClassVar[RenderMode]

class FontFamily:
    MONOSPACE: ClassVar[FontFamily]
    PROPORTIONAL: ClassVar[FontFamily]

class FontSize:
    numerator: int
    denominator: int
    EXTRA_SMALL: ClassVar[FontSize]
    SMALL: ClassVar[FontSize]
    MEDIUM: ClassVar[FontSize]
    LARGE: ClassVar[FontSize]
    EXTRA_LARGE: ClassVar[FontSize]
    FULL: ClassVar[FontSize]

    def __init__(self, numerator: int, denominator: int) -> None: ...

class TouchEvent:
    x: int
    y: int
    press_count: int
    release_count: int
    is_now_pressed: bool
    is_pressed: bool
    is_released: bool
    is_held: bool

def draw_pixel(x: int, y: int, color: Color) -> None: ...

def draw_line(start_x: int, start_y: int, end_x: int, end_y: int, color: Color) -> None: ...

def draw_circle(x: int, y: int, radius: int, color: Color) -> None: ...

def fill_circle(x: int, y: int, radius: int, color: Color) -> None: ...

def draw_rect(x: int, y: int, width: int, height: int, color: Color) -> None: ...

def fill_rect(x: int, y: int, width: int, height: int, color: Color) -> None: ...

def draw_buffer(x: int, y: int, width: int, height: int, buffer: Any) -> None: ...

def draw_text(text: str, x: int, y: int, *, font_size: FontSize = FontSize.MEDIUM, font_family: FontFamily = FontFamily.PROPORTIONAL, color: Color = Color.WHITE, bg_color: Color | None = None) -> None: ...

def print(*values: object, sep: str = ' ', end: str = '\n') -> None: ...

def scroll(start: int, offset: int) -> None: ...

def scroll_region(x: int, y: int, width: int, height: int, offset: int) -> None: ...

def set_render_mode(render_mode: RenderMode) -> None: ...

def render() -> None: ...

def erase(color: Color) -> None: ...

def get_touch_status() -> TouchEvent: ...

def is_now_pressed() -> bool: ...

def is_pressed() -> bool: ...

def is_released() -> bool: ...

def is_held() -> bool: ...
