from ovejas.system import User, Group
from ovejas.docker import Image
from ovejas.registry import ResourceRegistry
from ovejas.curl import Curl

for i in range(10):
    user = User(f'user_{i}', name=f"user_{i}", uid=4000 + i)
