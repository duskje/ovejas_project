from lib.registry import register

@register
class User:
    resource_name: str
    uid: int
    name: str

@register
class Group:
    resource_name: str
    name: str
    gid: int
