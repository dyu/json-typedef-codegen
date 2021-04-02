# Code generated by jtd-codegen for Python v0.3.1

import re
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Any, Dict, Optional, Union, get_args, get_origin


@dataclass
class RootNestedIDInitialism:
    json: 'str'
    normalword: 'str'

    @classmethod
    def from_json_data(cls, data: Any) -> 'RootNestedIDInitialism':
        return cls(
            _from_json_data(str, data.get("json")),
            _from_json_data(str, data.get("normalword")),
        )

    def to_json_data(self) -> Any:
        data: Dict[str, Any] = {}
        data["json"] = _to_json_data(self.json)
        data["normalword"] = _to_json_data(self.normalword)
        return data

@dataclass
class Root:
    http: 'str'
    id: 'str'
    nested_id_initialism: 'RootNestedIDInitialism'
    utf8: 'str'
    word_with_embedded_id_initialism: 'str'
    word_with_trailing_initialism_id: 'str'

    @classmethod
    def from_json_data(cls, data: Any) -> 'Root':
        return cls(
            _from_json_data(str, data.get("http")),
            _from_json_data(str, data.get("id")),
            _from_json_data(RootNestedIDInitialism, data.get("nested_id_initialism")),
            _from_json_data(str, data.get("utf8")),
            _from_json_data(str, data.get("word_with_embedded_id_initialism")),
            _from_json_data(str, data.get("word_with_trailing_initialism_id")),
        )

    def to_json_data(self) -> Any:
        data: Dict[str, Any] = {}
        data["http"] = _to_json_data(self.http)
        data["id"] = _to_json_data(self.id)
        data["nested_id_initialism"] = _to_json_data(self.nested_id_initialism)
        data["utf8"] = _to_json_data(self.utf8)
        data["word_with_embedded_id_initialism"] = _to_json_data(self.word_with_embedded_id_initialism)
        data["word_with_trailing_initialism_id"] = _to_json_data(self.word_with_trailing_initialism_id)
        return data

def _from_json_data(cls: Any, data: Any) -> Any:
    if data is None or cls in [bool, int, float, str, object] or cls is Any:
        return data
    if cls is datetime:
        return _parse_rfc3339(data)
    if get_origin(cls) is Union:
        return _from_json_data(get_args(cls)[0], data)
    if get_origin(cls) is list:
        return [_from_json_data(get_args(cls)[0], d) for d in data]
    if get_origin(cls) is dict:
        return { k: _from_json_data(get_args(cls)[1], v) for k, v in data.items() }
    return cls.from_json_data(data)

def _to_json_data(data: Any) -> Any:
    if data is None or type(data) in [bool, int, float, str, object]:
        return data
    if type(data) is datetime:
        return data.isoformat()
    if type(data) is list:
        return [_to_json_data(d) for d in data]
    if type(data) is dict:
        return { k: _to_json_data(v) for k, v in data.items() }
    return data.to_json_data()

def _parse_rfc3339(s: str) -> datetime:
    datetime_re = '^(\d{4})-(\d{2})-(\d{2})[tT](\d{2}):(\d{2}):(\d{2})(\.\d+)?([zZ]|((\+|-)(\d{2}):(\d{2})))$'
    match = re.match(datetime_re, s)
    if not match:
        raise ValueError('Invalid RFC3339 date/time', s)

    (year, month, day, hour, minute, second, frac_seconds, offset,
     *tz) = match.groups()

    frac_seconds_parsed = None
    if frac_seconds:
        frac_seconds_parsed = int(float(frac_seconds) * 1_000_000)
    else:
        frac_seconds_parsed = 0

    tzinfo = None
    if offset == 'Z':
        tzinfo = timezone.utc
    else:
        hours = int(tz[2])
        minutes = int(tz[3])
        sign = 1 if tz[1] == '+' else -1

        if minutes not in range(60):
            raise ValueError('minute offset must be in 0..59')

        tzinfo = timezone(timedelta(minutes=sign * (60 * hours + minutes)))

    second_parsed = int(second)
    if second_parsed == 60:
        second_parsed = 59

    return datetime(int(year), int(month), int(day), int(hour), int(minute),
                    second_parsed, frac_seconds_parsed, tzinfo)            