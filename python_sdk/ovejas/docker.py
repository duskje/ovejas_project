from .registry import Resource, register

@register
class Image(Resource): 
    resource_name: str
    tag: str
    environment: dict[str, str]
    restart: bool = False
