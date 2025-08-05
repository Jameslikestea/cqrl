---
title: Events
nav_order: 1
---

# Events

CQRL emits and consumes cloud events, this is it's method of interacting with your backend services.

## Emitted Events

### Commands

Subject: `cqrl.command.<COMMAND_NAME>`

For each `command` that you create in your definition file, the service will emit one cloud event per request that is serviced by the API. The event will be in the format of

```json
{
  "specversion": "1.0",
  "id": "01K1XRZPBPPZBM1FVDPXBYPP8J",
  "type": "cqrl.command",
  "source": "urn:cqrl:operation:new_todo:01K1XRZPBPPZBM1FVDPXBYPP8J",
  "datacontenttype": "application/json",
  "time": "2025-08-05T18:48:01.654Z",
  "data": {
    "content": "Buy Milk"
  },
  "authtype": "app_user",
  "authid": "subject_id"
}
```

| Field           | Meaning                                                                  |
| --------------- | ------------------------------------------------------------------------ |
| specversion     | The cloudevents version, in CQRL this should always be 1.0               |
| id              | The unique ID of the event (this is not the ID of the subject item)      |
| type            | Always cqrl.command in this case                                         |
| source          | The URN of the event                                                     |
| datacontenttype | Always JSON here                                                         |
| time            | The time of the event                                                    |
| data            | The validated model that the server is passing on                        |
| authtype        | App User or Unauthenticated                                              |
| authid          | Only set if authtype is `app_user` represents the `sub` field of the JWT |

## Consumed Events

### Query Update

Subject: `cqrl.update.<QUERY_NAME>`

For each value model that you want the query to return, send a single event to the server on the subject. The model should contain the all the required information.

```json
{
  "specversion": "1.0",
  "id": "01K1XRZPC4DTKEEW7ESKZ6WEHV",
  "type": "posts",
  "source": "01K1XRZPBPPZBM1FVDPXBYPP8J",
  "datacontenttype": "application/json",
  "time": "2025-08-05T18:48:01.654Z",
  "data": {
    "content": "Buy milk"
  },
  "subject": "01K1XRZPBPPZBM1FVDPXBYPP8J"
}
```

| Field           | Meaning                                                             |
| --------------- | ------------------------------------------------------------------- |
| specversion     | The cloudevents version, in CQRL this should always be 1.0          |
| id              | The unique ID of the event (this is not the ID of the subject item) |
| type            | The name of the query in the definition file                        |
| source          | Your reference of what caused the event to fire                     |
| datacontenttype | Always JSON here                                                    |
| time            | The time of the event                                               |
| data            | The data to put into the database                                   |
| subject         | The ULID of the model to update in the database                     |

### Permission Update

Subject: `cqrl.permission`

For each value model that you want the query to return, send a single event to the server on the subject. The model should contain the all the required information.

```json
{
  "specversion": "1.0",
  "id": "01K1XRZPC4M8QM4NJW1AE4XNBF",
  "type": "posts",
  "source": "01K1XRZPBPPZBM1FVDPXBYPP8J",
  "datacontenttype": "application/json",
  "time": "2025-08-05T18:48:01.654Z",
  "data": {
    "level": "read",
    "type": "permit"
  },
  "subject": "01K1XRZPBPPZBM1FVDPXBYPP8J",
  "authid": "subject_id",
  "authtype": "app_user"
}
```

| Field           | Meaning                                                             |
| --------------- | ------------------------------------------------------------------- |
| specversion     | The cloudevents version, in CQRL this should always be 1.0          |
| id              | The unique ID of the event (this is not the ID of the subject item) |
| type            | The name of the query in the definition file                        |
| source          | Your reference of what caused the event to fire                     |
| datacontenttype | Always JSON here                                                    |
| time            | The time of the event                                               |
| data.level      | `read` or `write`                                                   |
| data.type       | `permit` or `deny`                                                  |
| subject         | The ULID of the model to update in the database                     |
| authtype        | Must be `app_user`                                                  |
| authid          | Set to the subject ID of the user the request relates to            |
