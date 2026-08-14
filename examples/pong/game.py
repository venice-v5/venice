from venice import TimeUnit, monotonic_time

SCREEN_WIDTH = 480
SCREEN_HEIGHT = 240

PADDLE_WIDTH = 9
PADDLE_HEIGHT = 52
PADDLE_MARGIN = 20
PADDLE_SPEED = 190.0

BALL_RADIUS = 6
BALL_SPEED = 190.0
BALL_ACCELERATION = 1.06
MAX_BALL_SPEED = 390.0
SERVE_VERTICAL_SPEEDS = (72.0, -105.0, 90.0, -64.0)


class Game:
    def __init__(self):
        self.left_y = SCREEN_HEIGHT / 2
        self.right_y = SCREEN_HEIGHT / 2
        self.left_score = 0
        self.right_score = 0
        self.paused = False
        self.reset_ball()

    def reset_match(self):
        self.left_score = 0
        self.right_score = 0
        self.paused = False
        self.left_y = SCREEN_HEIGHT / 2
        self.right_y = SCREEN_HEIGHT / 2
        self.reset_ball()

    def reset_ball(self):
        point = self.left_score + self.right_score
        direction = 1 if point % 2 == 0 else -1

        self.ball_x = SCREEN_WIDTH / 2
        self.ball_y = SCREEN_HEIGHT / 2
        self.ball_vx = BALL_SPEED * direction
        self.ball_vy = SERVE_VERTICAL_SPEEDS[point % len(SERVE_VERTICAL_SPEEDS)]
        self.serve_until = monotonic_time(TimeUnit.MILLIS) + 700

    def move_paddles(self, left_direction, right_direction, seconds):
        self.left_y = self._clamp_paddle(
            self.left_y + left_direction * PADDLE_SPEED * seconds
        )
        self.right_y = self._clamp_paddle(
            self.right_y + right_direction * PADDLE_SPEED * seconds
        )

    def update(self, seconds):
        if self.paused:
            return
        if monotonic_time(TimeUnit.MILLIS) < self.serve_until:
            return

        self.ball_x += self.ball_vx * seconds
        self.ball_y += self.ball_vy * seconds

        top = BALL_RADIUS
        bottom = SCREEN_HEIGHT - BALL_RADIUS
        if self.ball_y < top:
            self.ball_y = top
            self.ball_vy = abs(self.ball_vy)
        elif self.ball_y > bottom:
            self.ball_y = bottom
            self.ball_vy = -abs(self.ball_vy)

        left_face = PADDLE_MARGIN + PADDLE_WIDTH
        right_face = SCREEN_WIDTH - PADDLE_MARGIN - PADDLE_WIDTH
        if self.ball_vx < 0 and self.ball_x - BALL_RADIUS <= left_face:
            if self._hits(self.left_y) and self.ball_x > PADDLE_MARGIN:
                self.ball_x = left_face + BALL_RADIUS
                self._bounce(self.left_y, 1)
        elif (
            self.ball_vx > 0
            and self.ball_x + BALL_RADIUS >= right_face
            and self._hits(self.right_y)
            and self.ball_x < SCREEN_WIDTH - PADDLE_MARGIN
        ):
            self.ball_x = right_face - BALL_RADIUS
            self._bounce(self.right_y, -1)

        if self.ball_x < -BALL_RADIUS:
            self._score(right=False)
        elif self.ball_x > SCREEN_WIDTH + BALL_RADIUS:
            self._score(right=True)

    def _bounce(self, paddle_y, direction):
        offset = (self.ball_y - paddle_y) / (PADDLE_HEIGHT / 2)
        speed = min(abs(self.ball_vx) * BALL_ACCELERATION, MAX_BALL_SPEED)
        self.ball_vx = speed * direction
        self.ball_vy = offset * speed * 0.78

    def _score(self, right):
        if right:
            self.left_score += 1
        else:
            self.right_score += 1
        self.reset_ball()

    def _hits(self, paddle_y):
        return abs(self.ball_y - paddle_y) <= PADDLE_HEIGHT / 2 + BALL_RADIUS

    @staticmethod
    def _clamp_paddle(y):
        half = PADDLE_HEIGHT / 2
        return max(half, min(SCREEN_HEIGHT - half, y))
