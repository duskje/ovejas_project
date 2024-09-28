from typing import Any, Protocol, dataclass_transform

import json


def create_init(annotations: dict, defaults: dict):
    parameters = []
    variables = []

    for variable_name, variable_type in annotations.items():
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


@dataclass_transform()
class Resource:
    _registered_resources = []

    def __init__(self, cls: type):
        if 'resource_name' not in cls.__annotations__.keys():
            raise NotImplementedError("Class must define attribute 'resource_name'")

        defaults = {}

        for attribute, value in cls.__dict__.items():
           if attribute[:2] != '__':
               defaults[attribute] = value

        annotations = cls.__annotations__

        init_function = create_init(annotations, defaults)

        setattr(cls, '__init__', init_function)

        self.cls = cls

    def get_uri(self, resource_name: str) -> str:
        return f'{self.cls.__module__}::{self.cls.__name__}::{resource_name}'

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        object_instance: ResourceProtocol = self.cls(*args, **kwargs) # Cada recurso debe tener el atributo resource_name

        # Cada recurso instanciado debe ser único
        if object_instance.resource_name in Resource._registered_resources:
            raise ValueError(f"Cannot have two resources with the same 'resource_name' attribute ({object_instance.resource_name})")
        
        Resource._registered_resources.append({
            'uri': self.get_uri(object_instance.resource_name),
            'parameters': kwargs,
        })

        return object_instance

    @classmethod
    def get_all(cls):
        return {
            "version": 1,
            "resources": cls._registered_resources,
        }

@Resource
class User:
    resource_name: str
    uid: int
    name: str

@Resource
class Docker:
    resource_name: str
    tag: str
    restart: bool = False

if __name__ == '__main__':
    puerco_user = User('puerco_user', name='Hiver', uid=512)

    print(puerco_user.uid)

    User('perro_user', name='Akira', uid=256)

    Docker('no_me_la_container', tag='0.1.0')

    for i in range(8):
        User(f'user_{i}', name=str(i), uid=i)

    print(json.dumps(Resource.get_all(), indent=2))

