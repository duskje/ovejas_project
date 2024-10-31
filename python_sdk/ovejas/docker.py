from .registry import register

@register
class Image: 
    resource_name: str
    tag: str
    environment: dict[str, str]
    restart: bool = False
