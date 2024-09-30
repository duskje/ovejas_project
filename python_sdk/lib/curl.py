from dataclasses import dataclass, field
from typing import Any, Callable, Generic, Literal, Optional, Protocol, TypeVar, cast

import subprocess

from lib.registry import register

# class Resolvable(Protocol):
#     def resolve(self) -> Any: ...

@dataclass(frozen=True)
class CurlResponse:
    status_code: int

    def resolve(self):
        pass


T = TypeVar('T')

class Result(Generic[T]):
    def __init__(self, result: Optional[T] = None, error: Optional[str] = None):
        if result is None and error is None:
            raise ValueError('Both can´t be None')

        self.result = result
        self.error = error

    @classmethod
    def from_result(cls, result: T):
        return cls(result=result)

    @classmethod
    def from_error(cls, error: str):
        return cls(error=error)

    def __repr__(self):
        if self.result is not None:
            return f'Result(result={self.result.__repr__()})'

        if self.error is not None:
            return f'Result(error={self.error.__repr__()})'

        raise ValueError('Both can´t be None')

    def transform(self, func: Callable) -> 'Result':
        return Result(result=func(self.result))

def Resolve(cls):
    return field(init=False)

@register
class Curl:
    resource_name: str
    method: str
    url: Result[str] | str

    response: Result[int] = Resolve()

    def create(self):
        result = subprocess.run(['curl', '-I', self.url.value],
                                stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE)

        status_code = int(result.stdout.splitlines()[0].split()[1])

        setattr(self, 'status_code', Result.from_result(status_code))
        
if __name__ == '__main__':
    req = Curl('req', 'GET', 'https://www.google.com/')

    req.create()

    print(req.response.transform(lambda status_code: f'statuscode={status_code}').result)
