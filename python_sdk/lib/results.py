from dataclasses import dataclass, field
from typing import Any, Callable, Generic, Literal, Optional, Protocol, TypeVar, cast
from weakref import ReferenceType 

T = TypeVar('T')

class Resolvable(Protocol):
    @staticmethod
    def resolve(dependent) -> Any: ...

class Result(Generic[T]):
    def __init__(self, resolvable: type, dependent: Any = None):
        self.resolvable = resolvable
        self.dependent = dependent
    
    def set_dependent(self, dependent: Any):
        self.dependent = dependent

    def resolve(self) -> 'Result[T]':
        return Result(self.resolvable.resolve(self.dependent()), self.dependent)

    def __repr__(self):
        return f'Result(resolvable={self.resolvable.__name__}, dependent={self.dependent})'

def resolve(cls):
    return Result(cls)
