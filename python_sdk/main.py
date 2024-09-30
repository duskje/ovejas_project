from lib.system import User, Group
from lib.docker import Image
from lib.registry import ResourceRegistry

puerco_user = User('puerco_user', name='Hiver', uid=512)
User('puerco_user2', name='Hiver', uid=512)

Group('my_group', name='cotorras', gid=10)

print(puerco_user.uid)

# User('perro_user', name='Akira', uid=256)

Image('no_me_la_container', tag='0.1.0', environment={'USER_UID': str(puerco_user.uid)})

for i in range(8):
    User(f'user_{i}', name=f'user{i}', uid=i)

print(ResourceRegistry.as_json())
