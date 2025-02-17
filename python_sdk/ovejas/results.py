from abc import ABC
from dataclasses import dataclass, field
from collections.abc import Callable
from typing import TYPE_CHECKING, Any, Callable, Generic, Literal, Optional, Protocol, TypeVar, cast, runtime_checkable
from weakref import ReferenceType

if TYPE_CHECKING:
    from .registry import Resource

class Resolvable(ABC):
    @staticmethod
    def resolve(dependent: Any) -> Any:
        pass

T = TypeVar('T')

class Result(Generic[T]):
    def __init__(self, resolvable, _root_level: Optional[bool] = None, _apply_function = None):
        self._root_level = _root_level

        self.resolvable = resolvable
        self.dependent: Optional['Resource'] = None
        self.attribute_tag: Optional[str] = None

        def unit_function(obj):
            return obj

        self._apply_function: Callable = unit_function if _apply_function is None else _apply_function
    
    def set_dependent(self, dependent: 'Resource'):
        self.dependent = dependent

    def set_attribute_tag(self, attribute_tag: str):
        self.attribute_tag = attribute_tag

    @property
    def urn(self):
        if self._root_level:
            return self.resolvable.urn

        if self.dependent is None:
            raise ValueError("Dependent is not set")

        if self.attribute_tag is None:
            raise ValueError("Attribute tag is not set")

        return f'{self.dependent.urn}::{self.attribute_tag}'

    def resolve(self) -> 'Result[T]':
        if self._root_level:
            return self._apply_function(self.resolvable)

        if self.dependent is None:
            raise ValueError("Dependent is not set")

        return self.resolvable.resolve(self.dependent)

    def apply(self, function: Callable):
        return Result(self.resolvable, self._root_level, function)

    def __repr__(self):
        if self._root_level:
            return f'Result(resolvable={self.resolvable}, dependent=None)'

        return f'Result(resolvable={self.resolvable}, dependent={self.dependent})'

type TypeOrResult[T] = Result[T] | T

def resolvable(cls):
    return Result(cls)
