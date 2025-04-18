from ovejas.system import User, Group
from ovejas.docker import Image
from ovejas.registry import ResourceRegistry
from ovejas.curl import Curl

puerco_user = User('puerco_user', name='Hiver', uid=512)
User('puerco_user2', name='Hiver', uid=512)

Group('my_group', name='cotorras', gid=10)

print(puerco_user.uid)

# User('perro_user', name='Akira', uid=256)

Image('no_me_la_container', tag='0.1.0', environment={'USER_UID': str(puerco_user.uid)})

for i in range(4):
    User(f'user_{i}', name=f'user{i}', uid=i)

req = Curl('req', 'GET', 'https://www.google.com/')

print(ResourceRegistry.as_json())
