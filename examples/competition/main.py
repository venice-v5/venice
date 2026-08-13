from venice import *

# Create a competition template
comp = Competition()

# Driver routine
@comp.driver
async def driver():
    print("Driver control!")

# Autonomous routine
@comp.autonomous
async def auton():
    print("Autonomous!")

# Define your `async` entrypoint. This is where you should put your initialization logic (e.g. calibration)
async def main():
    await comp.run()

# Create an `async` runtime and start executing your `main` function
vasyncio.run(main())