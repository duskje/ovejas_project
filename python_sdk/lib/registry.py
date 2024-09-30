from dataclasses import Field
from typing import Any, Protocol, TypeVar, cast, dataclass_transform

import json
import os


def create_init(cls: type):
    defaults = {}

    ignored_attributes = []

    for attribute, value in cls.__dict__.items():
        if attribute[:2] == '__': # Filter dunder methods
            continue

        if isinstance(value, Field): # Ignore field attribute
            ignored_attributes.append(attribute)
            continue

        defaults[attribute] = value

    annotations = cls.__annotations__

    parameters = []
    variables = []

    for variable_name, variable_type in annotations.items():
        if variable_name in ignored_attributes:
            continue

        variables.append(variable_name)

        default_value = defaults.get(variable_name)

        if default_value is not None:
            parameters.append(f'{variable_name}: {variable_type.__name__} = {default_value}')
        else:
            parameters.append(f'{variable_name}: {variable_type.__name__}')

    function_prototype = f"def __init__(self, {', '.join(parameters)}):"
    function_body = [f'setattr(self, "{variable_name}", {variable_name})' for variable_name in variables]

    function_definition = '\n\t'.join([function_prototype] + function_body)

    exec(function_definition)

    function = locals().pop('__init__')

    return function

class ResourceProtocol(Protocol):
    @property
    def resource_name(self) -> str: ...


class ResourceRegistry:
    _registered_resources = []
    command = os.environ.get('__command')

    def __init__(self, cls):
        if 'resource_name' not in cls.__annotations__.keys():
            raise NotImplementedError(f"Class '{cls.__name__}' must define attribute 'resource_name'")

        cls.__init__ = create_init(cls)

        self.cls = cls

    def get_uri(self, resource_name: str) -> str:
        return f'{self.cls.__module__}::{self.cls.__name__}::{resource_name}'

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        object_instance: ResourceProtocol = self.cls(*args, **kwargs) # Each resource should have the attribute 'resource_name'

        # Check if urn is unique
        registered_urns = (r.get('urn') for r in ResourceRegistry._registered_resources)
        object_urn = self.get_uri(object_instance.resource_name)

        if object_urn in registered_urns:
            raise ValueError(f"Cannot have two resources with the same uri ({object_urn})")
        
        ResourceRegistry._registered_resources.append({
            'urn': object_urn,
            'parameters': kwargs,
        })

        return object_instance

    @classmethod
    def as_json(cls):
        return json.dumps({
            "version": 1,
            "command": cls.command,
            "resources": cls._registered_resources,
        }, indent=2)

T = TypeVar('T')

@dataclass_transform(frozen_default=True)
def register(cls: type[T]) -> type[T]:
    return cast(type[T], ResourceRegistry(cls))

