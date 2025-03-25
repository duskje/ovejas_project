from .registry import Resource, provider_namespace, register, NamespaceMetadata
from typing import Optional

metadata = NamespaceMetadata(
    namespace='core.system',
    schema='0.0.1',
)

@provider_namespace(metadata)
class User(Resource):
    resource_name: str
    uid: int
    name: str
    gid: Optional[int] = 0

@provider_namespace(metadata)
class Group(Resource):
    resource_name: str
    name: str
    gid: int
