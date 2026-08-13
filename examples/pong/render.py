"""Minimal double-buffered rendering for Pong."""

from game import (
    BALL_RADIUS,
    PADDLE_HEIGHT,
    PADDLE_MARGIN,
    PADDLE_WIDTH,
    SCREEN_WIDTH,
)
from venice import Color, display

BACKGROUND = Color(5, 9, 20)
LEFT_PADDLE = Color(28, 225, 255)
RIGHT_PADDLE = Color(255, 54, 153)


def configure_display():
    display.set_render_mode(display.RenderMode.DOUBLE_BUFFERED)


def draw(game):
    display.erase(BACKGROUND)
    right_x = SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH
    _draw_score(game.left_score, 4)
    _draw_score(game.right_score, SCREEN_WIDTH - 74)
    _draw_paddle(PADDLE_MARGIN, game.left_y, LEFT_PADDLE)
    _draw_paddle(right_x, game.right_y, RIGHT_PADDLE)
    display.fill_circle(int(game.ball_x), int(game.ball_y), BALL_RADIUS, Color.WHITE)
    display.render()


def _draw_score(score, x):
    display.draw_text(f"{score} points", x, 0)


def _draw_paddle(x, center_y, color):
    y = int(center_y - PADDLE_HEIGHT / 2)
    display.fill_rect(x, y, PADDLE_WIDTH, PADDLE_HEIGHT, color)
