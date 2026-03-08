from venice import Motor, Gearset, Direction
"""This file is where you will define your drivetrain
This varible down here will define what drivetrain you will use,
The options for drive trains right now are:
    Tank Drive
    In the future we will add:
    Mechanum
    Tank Drive 2 Motor
    X-Drive

    Copy and paste the name of your drive train from the options into the variable
    Names of drivetrains:
        tank
        x_drive
        mechanum

    Ex:
    DriveTrainType = "tank" """
DriveTrainType = "Null"

"""Type how many motors you have in these lists, it goes from front to back!
Only 11 wats thouugh"""
LeftMotors: list = [0, 0]
RightMotors: list = [0, 0]

"""You are done! Keep reading if you are curious how the code functions!"""

def create_motor(port: int) -> Motor:

    direction = Direction.FORWARD
    port_abs: int = abs(port)

    if port < 0:
        return Motor(port_abs, Direction.REVERSE, Gearset.GREEN)
    else:
        return Motor(port_abs, Direction.FORWARD, Gearset.GREEN)

def TankDrive(LeftMotors, RightMotors):
    LeftMotorsList: list = []

    for motors in LeftMotors:
        LeftMotorsList.append(create_motor(LeftMotors[motors]))


        RightMotorsList: list = []

        for motors in RightMotors:
            RightMotorsList.append(create_motor(RightMotors[motors]))

    return LeftMotors, RightMotors
    def MoveForward(LeftMotors, RightMotors, Velocity)
        NegativeVeloctiy = Velocity*-1
        for motor in LeftMotors:
            motor.set_voltage(Velocity)
        for motor in RightMotors:
            motor.set_voltage(Velocity)


    def Turn(LetMotors, RightMotors, Velocity)
        for motor in LeftMotors:
            motor.set_voltage(Velocity)
        for motor in RightMotors:
            motor.set_velocity(NegativeVelocity)


    def SLIGHT_TURN(LeftMotors, RightMotors, Velocity)
        for motor in LeftMotors:
            if 12 + Velocity > 12:
                motor.set_voltage(12)
            else:
                motor.set_voltage(Velocity + 12)
        for motor in RightMotors:
            for motor in LeftMotors:
                if 12 - Velocity > 12:
                    motor.set_voltage(12)
                else:
                    motor.set_voltage(Velocity - 12)








def DriveTrainSelector(LeftMotors, RightMotors):
    if DriveTrainType == "tank":
        TankDrive(LeftMotors, RightMotors)
    else:
        print("ded")
