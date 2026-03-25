"""
Venice is an open-source Micropython runtime for VEX V5 robots.

```python
from venice import Motor, Gearset, Direction
import venice.vasyncio

async def main():
    my_motor = Motor(
        1,
        Direction.FORWARD,
        Gearset.GREEN
    )
    my_motor.set_voltage(10.0)
vasyncio.run(main())
```
"""

from __future__ import annotations

from typing import ClassVar, List, Literal, Union, Never

###########################################
# Binary file provider                    #
# Stuff for the CLI. DO NOT use in user   #
# code                                    #
###########################################
import importlib.resources as pkg_resources  # noqa: E402
import importlib.metadata  # noqa: E402

__version__: str
try:
    __version__ = importlib.metadata.version(__package__ if __package__ else "")
except importlib.metadata.PackageNotFoundError:
    __version__ = "0.1.0"

def _dangerous_DO_NOT_TOUCH_YOU_WILL_GET_ELECTROCUTED_get_binary_path():  # pyright: ignore[reportUnusedFunction]
    """DO NOT use this method in user programs that you are intending to run on the VEX V5.

    Internal function for use by the CLI."""
    return pkg_resources.files("venice").joinpath("venice.bin")
def _dangerous_DO_NOT_TOUCH_YOU_WILL_GET_ELECTROCUTED_get_version():  # pyright: ignore[reportUnusedFunction]
    """DO NOT use this method in user programs that you are intending to run on the VEX V5.

    Internal function for use by the CLI."""
    return __version__
