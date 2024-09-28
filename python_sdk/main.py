from typing import Any, dataclass_transform


def create_init(annotations: dict, defaults: dict):
    parameters = []

    for variable_name, variable_type in annotations.items():
        default_value = defaults.get(variable_name)

        if default_value is not None:
            parameters.append(f'{variable_name}: {variable_type.__name__} = {default_value}')
        else:
            parameters.append(f'{variable_name}: {variable_type.__name__}')

    function_definition = f"def __init__(self, {', '.join(parameters)}): pass"

    exec(function_definition)

    function = locals().pop('__init__')

    return function

@dataclass_transform()
class Resource(type):
    _registered_resources = []

    def __init__(cls, name: str, bases, attr_dict: dict):
        super().__init__(name, bases, attr_dict)

        defaults = {}

        for attribute, value in attr_dict.items():
            if attribute[:2] != '__':
                defaults[attribute] = value

        annotations = attr_dict['__annotations__']

        init_function = create_init(annotations, defaults)

        setattr(cls, '__init__', init_function)

    def get_uri(cls, resource_name: str) -> str:
        return f'{cls.__module__}::{cls.__name__}::{resource_name}'

    def __call__(cls, *args: Any, **kwargs: Any) -> Any:
        if len(args) < 1 and kwargs.get('resource_name') is None:
            raise ValueError('Expected a "resource_name" positional or keyword argument')

        if len(args) > 1:
            raise ValueError('Expected at most a single "resource_name" positional argument')

        if len(args) == 1:
            resource_name: str = args[0]

        if kwargs.get('resource_name') is not None:
            resource_name: str = kwargs.get('resource_name')

        Resource._registered_resources.append({
            'uri': cls.get_uri(resource_name),
            'parameters': kwargs,
        })

        return super().__call__(*args, **kwargs)


class User(metaclass=Resource):
    resource_name: str
    uid: int
    name: str

class Docker(metaclass=Resource):
    resource_name: str
    tag: str
    restart: bool = False

if __name__ == '__main__':
    User('puerco_user', name='Hiver', uid=512)
    User('perro_user', name='Akira', uid=256)
    Docker('no_me_la_container', tag='0.1.0')

    print(Resource._registered_resources)

