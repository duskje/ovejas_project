from collections.abc import Mapping
from dataclasses import Field, asdict, is_dataclass
from typing import Any, NotRequired, Optional, Protocol, TypeVar, TypedDict, cast, dataclass_transform, Union

import json
import os
import weakref
import datetime

from ovejas import _EXECUTION_CONTEXT

from .results import Result, TypeOrResult

if _EXECUTION_CONTEXT is None:
    raise RuntimeError("This project needs to be executed with the CLI tool")

def get_default_parameters(cls: type):
    defaults = {}

    for attribute, value in cls.__dict__.items():
        if isinstance(value, Result):
            continue

        if attribute[:2] == '__':
            continue

        if attribute[:1] == '_':
            continue

        if attribute == '_set_dependents':
            continue

        defaults[attribute] = value

    return defaults


def create_init(cls: type):
    defaults = {}

    ignored_attributes = []

    for attribute, value in cls.__dict__.items():
        if isinstance(value, Result):
            ignored_attributes.append(attribute)
            continue

        # Filter dunder methods
        if attribute[:2] == '__':
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

    print(function_definition)

    exec(function_definition)

    function = locals().pop('__init__')

    return function

type SchemaVersion = str;
type ResourceNamespace = str;

class NamespaceMetadata(TypedDict):
    schema: SchemaVersion
    namespace: ResourceNamespace
    
class Resource(Protocol):
    resource_name: str
    namespace_metadata: NamespaceMetadata

    @property
    def urn(self):
        return f'{type(self).__module__}::{type(self).__name__}::{self.resource_name}'


type Urn = str;

class RegisteredResource(TypedDict):
    urn: str
    parameters: Mapping[str, Any]
    depends_on: list[Urn]
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

    @staticmethod
    def inject_dependents_into_resolvables(object_instance: Any):
        resolvables = {}

        for attribute_key in object_instance.__dir__():
             attribute_value = getattr(object_instance, attribute_key)

             if isinstance(attribute_value, Result):
                 attribute_value.set_dependent(object_instance)
                 attribute_value.set_attribute_tag(attribute_key)

                 resolvables[attribute_key] = attribute_value

        depends_on = resolvables.values()

        return resolvables, depends_on

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        object_instance: Resource = self.cls(*args, **kwargs)
        parameters = get_default_parameters(self.cls)

        for key, value in kwargs.items():
            parameters[key] = value

        print('parameters', parameters)

        registered_urns = (r.get('urn') for r in ResourceRegistry._registered_resources)

        if object_instance.urn in registered_urns:
            raise ValueError(f"Cannot have two resources with the same uri ({object_instance.urn})")

        resolvables, depends_on  = self.inject_dependents_into_resolvables(object_instance)

        print('depends-on', depends_on)

        ResourceRegistry._registered_resources.append({
            'urn': object_instance.urn,
            'parameters': parameters,
            'depends_on': [dependent.urn for dependent in depends_on],
            'results': resolvables,
        })

        return object_instance

    @classmethod
    def as_json(cls):
        registered_resources = cls._registered_resources

        dependency_graph: Mapping[Urn, list[Urn]] = {}

        no_dependencies: list[Urn] = []

        for registered_resource in registered_resources:
            urn = registered_resource.get('urn')
            dependencies = registered_resource.get('depends_on')

            if dependencies is None:
                no_dependencies.append(urn)
                continue

            for dependency in dependencies:
                if dependency_graph.get(urn) is None:
                    dependency_graph[urn] = [dependency]
                else:
                    dependency_graph[urn].append(dependency)

        print('dependency_graph:', dependency_graph)
        print('no_dependencies:', no_dependencies)

        resolved_results = {}

        for registered_resource in registered_resources:
            results = registered_resource.get('results')

            print(registered_resource, results)

            if results:
                for unsolved_result_key, unsolved_result_value in results.items():
                    print('unsolved_result_value', unsolved_result_value.resolve())
                    resolved_results[unsolved_result_key] = asdict(unsolved_result_value.resolve())

                registered_resource['results'] = resolved_results
            else:
                print('not results', registered_resource, results)
                del registered_resource['results']

        return json.dumps({
            "version": 1,
            "created_at": str(datetime.datetime.now()),
            "resources": cls._registered_resources,
        }, indent=2)


T = TypeVar('T')

@dataclass_transform(frozen_default=True)
def register(cls: type[T]) -> type[T]:
    return cast(type[T], ResourceRegistry(cls))


def provider_namespace(metadata: NamespaceMetadata):
    @dataclass_transform(frozen_default=True)
    def register(cls: type[T]) -> type[T]:
        resource = cast(type[T], ResourceRegistry(cls))

        setattr(resource, 'namespace_metadata', metadata)

        return resource

    return register
