from .registry import Resource, register
from typing import Optional

resource_metadata = {
    "namespace": "core.system",
    "schema": "0.0.1",
}

@register
class User(Resource):
    resource_name: str
    uid: int
    name: str
    gid: Optional[int] = -1

@register
class Group(Resource):
    resource_name: str
    name: str
    gid: int
