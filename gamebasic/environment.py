"""Variablen-Scopes fuer GameBasic.

Eine Variable hat in GameBasic immer einen festen Typ (Pascal-strikt).
Wir speichern pro Scope ein Dict: name -> {"type": str, "value": object}.
"""
from .errors import GBRuntimeError


class Environment:
    def __init__(self, parent=None):
        self.parent = parent
        self.vars: dict = {}

    def declare(self, name: str, type_name: str, default):
        if name in self.vars:
            existing = self.vars[name]
            if existing.get("const"):
                raise GBRuntimeError(
                    f"'{name}' ist CONST und kann nicht erneut deklariert werden"
                )
            if existing["type"] != type_name:
                raise GBRuntimeError(
                    f"Variable '{name}' war als {existing['type'].upper()} deklariert, "
                    f"jetzt als {type_name.upper()} - Typkonflikt"
                )
            # Gleicher Typ: idempotent (z.B. DIM in Schleifenkoerper). Wert bleibt.
            return
        self.vars[name] = {"type": type_name, "value": default}

    def get_slot(self, name: str) -> dict:
        if name in self.vars:
            return self.vars[name]
        if self.parent:
            return self.parent.get_slot(name)
        raise GBRuntimeError(f"Variable '{name}' nicht deklariert (DIM fehlt?)")

    def get(self, name: str):
        return self.get_slot(name)["value"]

    def set(self, name: str, value):
        self.get_slot(name)["value"] = value

    def has(self, name: str) -> bool:
        if name in self.vars:
            return True
        if self.parent:
            return self.parent.has(name)
        return False
