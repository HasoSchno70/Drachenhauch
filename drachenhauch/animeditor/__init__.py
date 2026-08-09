"""Qt-freies Datenmodell + Code-Generierung des Animations-FSM-Editors `dhanim`.

`.gbanim` = JSON im Runtime-Format des `animfsm`-Moduls (params/states/
transitions/default). Zusatzfelder, die nur der Editor braucht (Knoten-`x`/`y`,
`sheet`/`frame_w`/`frame_h`/`scale`), ignoriert die Runtime beim Laden.
"""
from .document import (
    ANY_STATE,
    OPS,
    PARAM_TYPES,
    AnimDoc,
    Condition,
    History,
    Param,
    State,
    Transition,
    snap,
    unique_name,
)

__all__ = [
    "ANY_STATE",
    "OPS",
    "PARAM_TYPES",
    "AnimDoc",
    "Condition",
    "History",
    "Param",
    "State",
    "Transition",
    "snap",
    "unique_name",
]
