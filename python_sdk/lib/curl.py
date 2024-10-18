from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
import subprocess
from typing import Protocol, Any, cast
from dataclasses import dataclass

from lib.registry import ResourceRegistry, register, Resource
from lib.results import Result, resolvable, Resolvable, TypeOrResult


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
    url: TypeOrResult[str]

    response: Result[CurlResponse] = resolvable(CurlResponse)


if __name__ == '__main__':
    import time
    from concurrent.futures import ThreadPoolExecutor, as_completed

    req = Curl('req', 'GET', 'https://www.google.com/')

    print(req.urn)
    print(req.response.resolve())
    print(req.response.urn)

    def get_request(i: int):
        resource_name = f'req_{i}'

        req = Curl(resource_name, 'GET', 'https://www.google.com/')
        print(resource_name, "will sleep for", i, "seconds")
        time.sleep(i)
        print(req.response.dependent)
        print(resource_name, req.response.resolve())

    with ThreadPoolExecutor() as executor:
        futures = []
        seconds = 5
        for i in range(seconds):
            futures.append(executor.submit(get_request, seconds - i))

        for future in as_completed(futures):
            future.result()

    print(ResourceRegistry.as_json())
