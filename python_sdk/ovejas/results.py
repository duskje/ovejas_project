from abc import ABC
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Callable, Generic, Literal, Optional, Protocol, TypeVar, cast, runtime_checkable
from weakref import ReferenceType
import weakref

if TYPE_CHECKING:
    from .registry import Resource

T = TypeVar('T')


class Resolvable(ABC):
    @staticmethod
    def resolve(dependent: Any) -> Any:
        pass

class Result(Generic[T]):
    def __init__(self, resolvable: type):
        self.resolvable = resolvable
        # self.dependent: Optional[weakref.ref['Resource']] = None
        self.dependent: Optional['Resource']
        self.attribute_tag: Optional[str] = None
    
#    def set_dependent(self, dependent: weakref.ref['Resource']):
#        self.dependent = dependent
    def set_dependent(self, dependent: 'Resource'):
        self.dependent = dependent

    def set_attribute_tag(self, attribute_tag: str):
        self.attribute_tag = attribute_tag

    @property
    def urn(self):
        if self.dependent is None:
            raise ValueError("Dependent is not set")

#        dependent_instance = self.dependent()
#
#        if dependent_instance is None:
#            raise ValueError("Dependent is not set")

        if self.attribute_tag is None:
            raise ValueError("Attribute tag is not set")

        return f'{self.dependent.urn}::{self.attribute_tag}'

    def resolve(self) -> 'Result[T]':
        if self.dependent is None:
            raise ValueError("Dependent is not set")

        return self.resolvable.resolve(self.dependent)

    def __repr__(self):
        return f'Result(resolvable={self.resolvable}, dependent={self.dependent})'

type TypeOrResult[T] = Result[T] | T

def resolvable(cls):
    return Result(cls)
