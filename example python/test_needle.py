import cactus
from typing import Literal

@cactus.tool
def set_thermostat(temperature: int, mode: Literal["heat", "cool", "auto"] = "auto"):
    """Set the thermostat.

    Args:
        temperature: target temperature in Celsius
        mode: heating strategy to use
    """
    return {"temperature": temperature, "mode": mode}

@cactus.tool
def set_lights(room: str, on: bool, brightness: int = 100):
    """Turn a room's lights on or off and set brightness.

    Args:
        room: which room to control
        on: whether the lights are on
        brightness: brightness from 0 to 100
    """
    return {"room": room, "on": on, "brightness": brightness}

agent = cactus.Cactus(tools=[set_thermostat, set_lights])
for q in ["make it 21 and cool the room",
          "turn off the kitchen lights",
          "dim the living room to 30 percent",
          "what's the capital of France?"]:
    print(q, "->", agent.run(q))
