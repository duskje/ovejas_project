from collections.abc import Mapping
from dataclasses import Field, asdict, is_dataclass
from typing import Any, NotRequired, Optional, Protocol, TypeVar, TypedDict, cast, dataclass_transform, Union

import json
import os
import weakref

from ovejas import _EXECUTION_CONTEXT

from .results import Resolvable, Result, TypeOrResult

if _EXECUTION_CONTEXT is None:
    raise RuntimeError("This project needs to be executed with the CLI tool")

def create_init(cls: type):
    defaults = {}

    ignored_attributes = []

    for attribute, value in cls.__dict__.items():
        if isinstance(value, Result):
            ignored_attributes.append(attribute)
            continue

        if attribute[:2] == '__': # Filter dunder methods
            continue

        if attribute == '_set_dependents':
            ignored_attributes.append(attribute)
            continue

        defaults[attribute] = value

    annotations = cls.__annotations__

    parameters = []
    variables = []

    for variable_name, variable_type in annotations.items():
        if isinstance(variable_type, Result):
            continue

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


class Resource(Protocol):
    resource_name: str

    @property
    def urn(self):
        return f'{type(self).__module__}::{type(self).__name__}::{self.resource_name}'


class RegisteredResource(TypedDict):
    urn: str
    parameters: Mapping[str, Any]
    results: NotRequired[Mapping[str, Any]]


class ResourceRegistry:
    _registered_resources: list[RegisteredResource] = []
    _dependency_graph = {}

    def __init__(self, cls):
        if 'resource_name' not in cls.__annotations__.keys():
            raise NotImplementedError(f"Class '{cls.__name__}' must define attribute 'resource_name'")

        cls.__init__ = create_init(cls)

        self.cls = cls

    def get_uri(self, resource_name: str) -> str:
        return f'{self.cls.__module__}::{self.cls.__name__}::{resource_name}'

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        object_instance: Resource = self.cls(*args, **kwargs) # Each resource should have the attribute 'resource_name'

        # Check if urn is unique
        registered_urns = (r.get('urn') for r in ResourceRegistry._registered_resources)
        object_urn = self.get_uri(object_instance.resource_name)

        # Result resolvables
        resolvables = {}

        for attribute_key in object_instance.__dir__():
             attribute_value = getattr(object_instance, attribute_key)

             if isinstance(attribute_value, Result):
                 # attribute_value.set_dependent(weakref.ref(object_instance))
                 attribute_value.set_dependent(object_instance)
                 attribute_value.set_attribute_tag(attribute_key)

                 resolvables[attribute_key] = attribute_value

        # Expect resolvables
        # Si le entra un result, esta instancia es dependendiente del recurso del cual el resultado es dueño
        for arg in args:
            pass

        if object_urn in registered_urns:
            raise ValueError(f"Cannot have two resources with the same uri ({object_urn})")

        ResourceRegistry._registered_resources.append({
            'urn': object_urn,
            'parameters': kwargs,
            'results': resolvables,
        })

        return object_instance

    @staticmethod
    def resolve_dependency_graph():
        for resource in ResourceRegistry._registered_resources:
             resource_urn = resource['urn']
             pass

    @classmethod
    def as_json(cls):
        registered_resources = cls._registered_resources

        resolved_results = {}

        for registered_resource in registered_resources:
            results = registered_resource.get('results')

            print(registered_resource, results)

            if results:
                for unsolved_result_key, unsolved_result_value in results.items():
                    resolved_results[unsolved_result_key] = asdict(unsolved_result_value.resolve())

                registered_resource['results'] = resolved_results
            else:
                print(registered_resource, results)
                del registered_resource['results']

        return json.dumps({
            "version": 1,
            "resources": cls._registered_resources,
        }, indent=2)

T = TypeVar('T')

@dataclass_transform(frozen_default=True)
def register(cls: type[T]) -> type[T]:
    return cast(type[T], ResourceRegistry(cls))

