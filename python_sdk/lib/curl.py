import subprocess
from typing import Protocol, Any, cast
from dataclasses import dataclass

from lib.registry import register, Resource
from lib.results import Result, resolve, Resolvable

import weakref

@dataclass
class CurlResponse(Resolvable):
    status_code: int

    @staticmethod
    def resolve(dependent: 'Curl') -> 'CurlResponse':
        result = subprocess.run(['curl', '-I', cast(str, dependent.url)],
                                stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE)

        status_code = int(result.stdout.splitlines()[0].split()[1])
        return CurlResponse(status_code=status_code)
    
@register
class Curl(Resource):
    resource_name: str
    method: str
    url: Result[str] | str

    response: Result[CurlResponse] = resolve(CurlResponse)

    def _set_dependents(self):
        self.response.set_dependent(weakref.ref(self))

if __name__ == '__main__':
    req = Curl('req', 'GET', 'https://www.google.com/')

    print(req.response)
