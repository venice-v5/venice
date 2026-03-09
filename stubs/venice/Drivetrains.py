from venice import Direction, Gearset, Motor

"""This file is where you will define your drivetrain
The variable DriveTrainType is what defines what dirve train you have, the current drivetrains that are
supported are these (Name followed for the variable in paranthesis)
Tank/Differential any motors (tank)
X - drive 4 Motors (holomonic)
Mecanum 4 motors (holomonic)
The math behind X drive and mecanum 4 motors is the same so we just call them the same thing! """

DriveTrainType = "Null"

"""Type how many motors you have in these lists, it goes from front to back! """
LeftMotors: list = [0, 0]
RightMotors: list = [0, 0]

# The way we define drivetrains is that each type of drivetrain is a class
# At the end, we declare our variable called "drivetrain" to be one of these classes
#   That depends on what the user puts in
# Each drivetrain contaitns the logic on how to move given Forward velocity, and how much you want to turn


def create_motor(port: int) -> Motor:
    port_abs: int = abs(port)
    if port < 0:
        return Motor(port_abs, Direction.REVERSE, Gearset.GREEN)
    else:
        return Motor(port_abs, Direction.FORWARD, Gearset.GREEN)


class TankDrivetrain:
    def __init__(self, left_ports: list[int], right_ports: list[int]):
        self.left_motors = [create_motor(p) for p in left_ports]
        self.right_motors = [create_motor(p) for p in right_ports]

    def move(self, velocity, turn: int):
        # These for loops just say for each motor on each side, move the motor however fast forward
        # and how fast you want to turn. This means regardless of our drivetrain type,
        #  we can call the same function!
        for m in self.left_motors:
            m.set_voltage(velocity + turn)
        for m in self.right_motors:
            m.set_voltage(velocity - turn)


class HolomonicDrive:
    def __init__(self, left_ports: list[int], right_ports: list[int]):
        self.LeftFront = create_motor(left_ports[0])
        self.LeftBack = create_motor(left_ports[1])
        self.RightFront = create_motor(right_ports[0])
        self.RightBack = create_motor(right_ports[1])

    def move(self, Forward, Rotate):
        FL = max(-12, min(12, Forward + Rotate))
        FR = max(-12, min(12, Forward - Rotate))
        BL = max(-12, min(12, Forward + Rotate))
        BR = max(-12, min(12, Forward - Rotate))

        self.LeftFront.set_voltage(FL)
        self.RightFront.set_voltage(FR)
        self.LeftBack.set_voltage(BL)
        self.RightBack.set_voltage(BR)

    def moveLateral(self, Forward, Lateral, Rotate):
        FL = max(-12, min(12, Forward + Lateral + Rotate))
        FR = max(-12, min(12, Forward - Lateral - Rotate))
        BL = max(-12, min(12, Forward - Lateral + Rotate))
        BR = max(-12, min(12, Forward + Lateral - Rotate))

        self.LeftFront.set_voltage(FL)
        self.RightFront.set_voltage(FR)
        self.LeftBack.set_voltage(BL)
        self.RightBack.set_voltage(BR)


class HolomonicDrive6:
    def __init__(self, left_ports: list[int], right_ports: list[int]):
        self.LeftFront = create_motor(left_ports[0])
        self.LeftMiddle = create_motor(left_ports[1])
        self.LeftBack = create_motor(left_ports[2])
        self.RightFront = create_motor(right_ports[0])
        self.RightMiddle = create_motor(right_ports[1])
        self.RightBack = create_motor(right_ports[2])

    def move(self, Forward, Rotate):
        FL = max(-12, min(12, Forward + Rotate))
        FR = max(-12, min(12, Forward - Rotate))
        ML = max(-12, min(12, Forward + Rotate))
        MR = max(-12, min(12, Forward - Rotate))
        BL = max(-12, min(12, Forward + Rotate))
        BR = max(-12, min(12, Forward - Rotate))

        self.LeftFront.set_voltage(FL)
        self.RightFront.set_voltage(FR)
        self.LeftMiddle.set_voltage(ML)
        self.RightMiddle.set_voltage(MR)
        self.LeftBack.set_voltage(BL)
        self.RightBack.set_voltage(BR)

    def move_Lateral(self, Forward, Lateral, Rotate):
        FL = max(-12, min(12, Forward + Lateral + Rotate))
        FR = max(-12, min(12, Forward - Lateral - Rotate))
        ML = max(-12, min(12, Forward + Rotate))
        MR = max(-12, min(12, Forward - Rotate))
        BL = max(-12, min(12, Forward - Lateral + Rotate))
        BR = max(-12, min(12, Forward + Lateral - Rotate))

        self.LeftFront.set_voltage(FL)
        self.RightFront.set_voltage(FR)
        self.LeftMiddle.set_voltage(ML)
        self.RightMiddle.set_voltage(MR)
        self.LeftBack.set_voltage(BL)
        self.RightBack.set_voltage(BR)


if DriveTrainType == "tank":
    drivetrain = TankDrivetrain(LeftMotors, RightMotors)
elif DriveTrainType == "holomonic":
    drivetrain = HolomonicDrive(LeftMotors, RightMotors)
elif DriveTrainType == "holomonic6":
    drivetrain = HolomonicDrive6(LeftMotors, RightMotors)
else:
    print("ded")
