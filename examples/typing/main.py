from typing import Union


my_var: Union[int, float] = "str"

print("Here's a cool type: ", Union[str, float])

"""
Expected output:

    Here's a cool type: typing.Union[str, float]

Note that it will really return smth weird like <_AnyCall object at 3a58ef0> but its fine because users shouldn't be printing types anyways lol.
This was mainly a test to ensure our bytecode injection of `typing` worked properly, which it did.
"""
