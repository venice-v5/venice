from venice import Motor, Gearset, Direction
"""This file is where you will define your drivetrain
This varible down here will define what drivetrain you will use,
The options for drive trains right now are:
    Tank Drive 6 Motor
    Tank Drive 4 Motor
    In the future we will add:
    Mechanum
    Tank Drive 2 Motor
    X-Drive

    Copy and paste the name of your drive train from the options into the variable
    Ex:
    DriveTrainType = "Tank Drive 6 Motor" """
DriveTrainType = "Null"

""" This next section will be defining your motors for your drives are
Just type in the port number for each of the motors,
make it a negative number for ports that are reversed
Set it to 0 if you do not use that motor"""
TopLeft = 0
MiddleLeft = 0
BottomLeft = 0
TopRight = 0
MiddleRight = 0
BottomRight = 0

"""You are done! Keep reading if you are curious how the code functions!"""

def create_motor(port: int) -> Motor:

    direction = Direction.FORWARD
    port_abs: int = abs(port)

    if port < 0:
        return Motor(port_abs, Direction.REVERSE, Gearset.GREEN)
    else:
        return Motor(port_abs, Direction.FORWARD, Gearset.GREEN)

def TankDrive6(TopLeft, MiddleLeft, BottomLeft, TopRight, MiddleRight, BottomRight):
    LeftMotors: list[Motor] =  [
        create_motor(TopLeft),
        create_motor(MiddleLeft),
        create_motor(BottomLeft)
    ]
    RightMotors: list[Motor] = [
        create_motor(TopRight),
        create_motor(MiddleRight),
        create_motor(BottomRight)
    ]
    return LeftMotors, RightMotors



    def MoveForward(LeftMotors, RightMotors, Velocity):
        for motor in LeftMotors:
            motor.set_voltage(Velocity)
        for motor in RightMotors:
            motor.set_voltage(Velocity)





def DriveTrainSelector(TopLeft, MiddleLeft, BottomLeft, TopRight, MiddleRight, BottomRight):
    if DriveTrainType == "Tank Drive 6 Motor":
