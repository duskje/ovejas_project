from typing import Any, Protocol, dataclass_transform

import json


def create_init(cls: type):
    defaults = {}

    for attribute, value in cls.__dict__.items():
       if attribute[:2] != '__':
           defaults[attribute] = value

    annotations = cls.__annotations__

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

class ResourceRegistry:
    _registered_resources = []

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
    def get_all(cls):
        return {
            "version": 1,
            "resources": cls._registered_resources,
        }

@dataclass_transform()
def register(cls):
    return ResourceRegistry(cls)
    
@register
class User:
    resource_name: str
    uid: int
    name: str

@register
class Docker: # TODO: change to 'Image'
    resource_name: str
    tag: str
    environment: dict[str, str]
    restart: bool = False

@register
class Curl:
    resource_name: str
    method: str
    url: str
    response_status_code: int


if __name__ == '__main__':
    puerco_user = User('puerco_user', name='Hiver', uid=512)

    print(puerco_user.uid) # Se instancia un objeto

    User('perro_user', name='Akira', uid=256)

    Docker('no_me_la_container', tag='0.1.0', environment={'USER_UID': str(puerco_user.uid)})

    for i in range(8):
        User(f'user_{i}', name=str(i), uid=i)

    print(json.dumps(ResourceRegistry.get_all(), indent=2))

