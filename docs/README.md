# CQRL

## What is CQRL?

CQRL is an advanced API intermediary that bridges the gap between client applications communicating via HTTP(S) and backend services operating on message queues. Unlike traditional API gateways that simply route requests, CQRL provides a complete asynchronous runtime environment for modern SaaS applications.

## Key Features

### Asynchronous Runtime

CQRL delivers a robust asynchronous runtime designed specifically for SaaS applications. It enables seamless communication between synchronous client requests and asynchronous backend services, allowing for more scalable and resilient system architectures.

### Security-First Design

- **Comprehensive Authentication & Authorization**: When configured, CQRL enforces strict AuthN and AuthZ protocols, ensuring that only authorized users can access specific resources.
- **Information Control**: CQRL prevents unintended data exposure by strictly adhering to defined schemas, returning only the specified data to clients.

### Message Queue Integration

CQRL seamlessly connects HTTP(S) clients with backend services listening on message queues, enabling event-driven architectures without requiring clients to implement messaging protocols.

## How It Works

CQRL sits as a mediator between your frontend applications and backend services:

1. Clients send standard HTTP(S) requests to CQRL
2. CQRL validates requests against schemas and security policies
3. Valid requests are transformed into messages for appropriate backend services
4. Responses from backend services are collected, validated, and returned to clients

This decoupled architecture allows teams to build more maintainable, scalable systems while providing a simple interface for client applications.

## Getting Started

[Installation and usage instructions coming soon]

## Documentation

For more detailed documentation, please visit our [documentation site](https://jameslikestea.github.io/cqrl/).

## Roadmap

- Features
    - Core
        - [x] Serve requests based on a CQRL service file
        - [x] Authenticate requests using JWKS and [Authcontext](https://github.com/cloudevents/spec/blob/main/cloudevents/extensions/authcontext.md)
        - [x] Validate requests against a schema
        - [x] OTLP Logging and metrics
        - [x] Authorize requests based on the Authcontext
        - [x] Enable rate limiting
            - [x] IP Address
            - [x] Auth Context
        - [ ] Enable websocket watch for processing status (Received, ACK'd, Completed, Failed)
        - [x] URL Parameters
            - [x] Pagination
            - [x] Get single item from the server
            - [x] Specify update subject
    - Generators
        - [x] Generate OpenAPI spec from CQRL service file
        - [x] Example for generating clients
- Documentation
    - Usage
        - [x] Add documentation for getting started
        - [ ] Add documentation for downstream services
        - [ ] Add documentation for how to update a model
        - [ ] Add documentation for how to authorize user access to models

## License

[License information](../LICENSE)

## Changelog

[Changelog](./CHANGELOG.md)
