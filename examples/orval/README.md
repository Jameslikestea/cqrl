# Orval

This example shows the example of integrating CQRL with Orval, an openapi generator for typescript projects. This version generates a svelte query type, but this can be used for any orval config.

## Using the example

Run the following commands to re-generate the ./src directory

```
cqrl generate openapi ./service.hcl ./service.openapi.json
orval
```

This generates a new service.openapi.json file and then generates the typescript source from it.