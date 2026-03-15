# Oshi API

API for upcoming streams of my oshi.

## Usage

### Get a list of all upcoming streams

```
curl oshi.killbasa.com
```

### Get the upcoming streams of a specific VTuber

A list of aliases can be found at `/list`.

```
curl oshi.killbasa.com?oshi=<alias>
```

### Get a list of available VTubers

```
curl oshi.killbasa.com/list
```

## Response formats

This API supports JSON and plain text responses. If you specify `Accept: application/json` in the request header the response will be in JSON. Otherwise, it will be in plain text.
