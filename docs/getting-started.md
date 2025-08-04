---
title: Getting Started
nav_order: 0
---

# Getting Started

It's super easy to get started with with CQRL, there are only a small number of dependencies. The docker-compose in the root of this project creates a development environment that can be used for testing, but these are not production ready. You should use managed or self-managed services that are configured for a production environment in production.

> CQRL has not been tested for production and is provided as-is with no warranty, usage in production is at your own risk.

## Dependencies

* NATS Cluster
* MongoDB Cluster (in a clustered mode, this is needed for watching the collection)
* JWKS Endpoint

## Defining your first API

To define your API you need to define a couple of things in your HCL file. The first is the models that you want to receive or return. Models are defined using the following syntax.

```hcl
model "todo" {
    content = {
        type = "string"
        required = true
    }
}
```

We then need to define the query, all the parts of the software centre around the query, with commands also being authorized through queries.

```hcl
query "my_todos" {
    modelled_by = model.todo
}
```

Finally we want to define our commands, we want two commands here

```hcl
command "new_todo" {
    modelled_by = model.todo
}

command "update_todo" {
    modelled_by = model.todo
    authorized_by = query.my_todos
}
```

All together this service file `service.hcl` will look something like this:

```hcl
model "todo" {
    content = {
        type = "string"
        required = true
    }
}

query "my_todos" {
    modelled_by = model.todo
}

command "new_todo" {
    modelled_by = model.todo
}

command "update_todo" {
    modelled_by = model.todo
    authorized_by = query.my_todos
}
```

## Generating Clients

We use openapi as an intermediary format for generating clients for the APIs that we're writing, these can be generated with the following command:

```bash
cqrl generate openapi ./service.hcl ./service.openapi.json
```

## Serving the API

To serve the API that we've just written we can run something like

```bash
cqrl serve --database-mode mongodb --mongodb-address "<CONNECTION_STRING>" --nats-address "<NATS_ADDRESS>" --jwks-endpoint "<JWKS_ENDPOINT>"./service.hcl
```

This will start serving the API on `0.0.0.0:8912`