export default {
    'cqrl-file': {
        input: './service.openapi.json',
        output: {
            target: './src/cqrl.ts',
            client: 'svelte-query',
            mode: 'tags-split',
            schemas: './src/models',
        },
    },
};